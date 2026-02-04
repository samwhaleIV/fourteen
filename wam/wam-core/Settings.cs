using SkiaSharp;

namespace WAM.Core {
    public record class TexturePackSettings(
        int MaxSize = 512,
        bool AllowMultipleSurfaces = true,
        PackPadding Padding = PackPadding.EdgeExtension
    );

    public record class WamManifestSettings(
        string Source,
        string Destination,
        string TargetNamespace,
        ImageFormat ImageExportFormat = ImageFormat.Png,
        TexturePackSettings? TexturePackSettings = null,
        bool UseGuids = true,
        bool CompressManifest = false,
        string ManifestOutputFile = "manifest.json"
    );

    public enum PackPadding {
        None,
        TransparentBuffer,
        EdgeExtension
    }

    public enum ImageFormat {
        Png,
        Webp
    }

    public static class ImageFormatExtensions {
        public static SKEncodedImageFormat ToSkFormat(this ImageFormat format) {
            return format switch {
                ImageFormat.Png => SKEncodedImageFormat.Png,
                ImageFormat.Webp => SKEncodedImageFormat.Webp,
                _ => throw new NotImplementedException(),
            };
        }
        public static string GetFileExtension(this ImageFormat format) {
            return format switch {
                ImageFormat.Png => ".png",
                ImageFormat.Webp => ".webp",
                _ => throw new NotImplementedException(),
            };
        }
    }
}
