namespace WAM.Core.Builder.TexturePack {
    internal sealed class LayoutSurfaceGenerator(TexturePackSettings settings):List<LayoutSurface> {

        public string RuntimeFileName { get; set; } = string.Empty;
        public string Namespace { get; set; } = string.Empty;
        public WamManifest? AssetGenerator { get; set; }

        public void AddGenerated() {
            var id = AssetGenerator?.BindAsset(
                RuntimeFileName,
                Namespace,
                string.Empty,
                settings.ExportFormat.GetFileExtension(),
                FileType.Image
            ) ?? 0;
            var layoutSurface = new LayoutSurface(settings.MaxSize,id);
            Add(layoutSurface);
        }
    }
}
