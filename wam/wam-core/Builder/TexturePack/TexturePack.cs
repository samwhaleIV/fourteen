using SkiaSharp;

namespace WAM.Core.Builder.TexturePack {

    public readonly record struct TexturePack(
        VirtualImageAsset[] Images,
        GeneratedFile[] Files
    );

    public sealed class TexturePackBuilder(TexturePackSettings settings) {

        private readonly List<string> imagePaths = [];
        private readonly List<GeneratedFile> generatedFiles = [];
        private readonly List<VirtualImageAsset> virtualImageFiles = [];
        private readonly List<Image> images = [];

        private readonly LayoutSurfaceGenerator layoutSurfaces = new(settings);

        public void Reset() {
            imagePaths.Clear();
            generatedFiles.Clear();
            virtualImageFiles.Clear();
        }

        public void AddImage(string file) {
            imagePaths.Add(file);
        }

        private readonly record struct Image(SKBitmap Bitmap,string FilePath) {
            public int GetLargestDimension() {
                return Math.Max(Bitmap.Width,Bitmap.Height); // Might want to change to just width
            }
        }

        private void DisposeSKObjects() {
            foreach(var image in images) {
                image.Bitmap.Dispose();
            }
            foreach(var layoutSurface in layoutSurfaces) {
                layoutSurface.Dispose();
            }
            layoutSurfaces.Clear();
            images.Clear();
        }

        private Result<TexturePack> InnerBuild(string runtimeFileName,string @namespace,WamManifest assetGenerator) {

            foreach(var filePath in imagePaths) {
                using var fileStream = File.OpenRead(filePath);
                try {
                    var skBitmap = SKBitmap.Decode(fileStream);
                    if(skBitmap != null) {
                        images.Add(new Image(skBitmap,filePath));
                    }
                } catch(Exception exception) {
                    return Result<TexturePack>.Err(exception.Message);
                }
            }

            images.Sort((b,a) => a.GetLargestDimension().CompareTo(b.GetLargestDimension()));

            layoutSurfaces.RuntimeFileName = runtimeFileName;
            layoutSurfaces.Namespace = @namespace;
            layoutSurfaces.AssetGenerator = assetGenerator;

            layoutSurfaces.AddGenerated();

            bool secondAttempt = false;
            for(int i = 0;i < images.Count;i++) {
                var image = images[i];

                bool success = false;
                for(int j = secondAttempt ? layoutSurfaces.Count - 1 : 0;j < layoutSurfaces.Count;j++) {
                    var surface = layoutSurfaces[j];
                    if(surface.TryAddBitmap(image.Bitmap,out Area area)) {
                        virtualImageFiles.Add(new() {
                            Area = new Area(),
                            Name = Path.Combine(
                                runtimeFileName,
                                Path.GetFileNameWithoutExtension(image.FilePath)
                            ),
                            ID = surface.ID
                        });
                        success = true;
                        break;
                    }
                }
                if(success) {
                    secondAttempt = false;
                    continue;
                } else {
                    if(secondAttempt) {
                        return Result<TexturePack>.Err(
                            $"texture pack item '{image.FilePath}' is too big for pack (size: {image.Bitmap.Width}x{image.Bitmap.Height})"
                        );
                    }

                    secondAttempt = true;
                    layoutSurfaces.AddGenerated();
                }
            }

            foreach(var surface in layoutSurfaces) {
                byte[] data;
                var destination = assetGenerator.GetAssetDestination(surface.ID);
                try {
                    data = surface.ExportPNG(settings.ExportFormat);
                } catch(Exception exception) {
                    return Result<TexturePack>.Err($"could not create pack for '{destination}': {exception.Message}");
                }
                if(data.Length < 1) {
                    return Result<TexturePack>.Err($"could not create pack for '{destination}': no canvas surface");
                }
                generatedFiles.Add(new() {
                    Destination = destination,
                    Data = data
                });
            }

            return Result<TexturePack>.Ok(new() {
                Images = [.. virtualImageFiles],
                Files = [.. generatedFiles]
            });
        }

        public Result<TexturePack> Build(string runtimeFileName,string @namespace,WamManifest assetGenerator) {
            DisposeSKObjects();
            return InnerBuild(runtimeFileName,@namespace,assetGenerator);
        }
    }
}
