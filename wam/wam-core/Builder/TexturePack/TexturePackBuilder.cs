namespace WAM.Core.Builder.TexturePack {
    public sealed class TexturePackBuilder(TexturePackSettings settings) {

        private readonly TexturePackSettings settings = settings;

        private readonly List<string> imagePaths = [];
        private readonly List<GeneratedFile> generatedFiles = [];
        private readonly List<VirtualImageFile> virtualImageFiles = [];

        public void Reset() {
            imagePaths.Clear();
            generatedFiles.Clear();
            virtualImageFiles.Clear();
        }

        public void AddImage(string file) {
            imagePaths.Add(file);
        }

        public Result<TexturePack> Build(string runtimeFileName,string @namespace,WamManifest assetGenerator) {
            var id = assetGenerator.BindAsset(
                runtimeFileName,
                @namespace,
                string.Empty,
                ".png",
                FileType.Image,
                []
            );
            foreach(var file in imagePaths) {
                virtualImageFiles.Add(new() {
                    Area = Area.Zero,
                    Type = FileType.Image.ToString(),
                    Name = Path.Combine(runtimeFileName,Path.GetFileNameWithoutExtension(file)),
                    ID = id
                });
            }
            return Result<TexturePack>.Ok(new TexturePack() {
                Images = [.. virtualImageFiles],
                Files = [.. generatedFiles]
            });
        }


    }
}
