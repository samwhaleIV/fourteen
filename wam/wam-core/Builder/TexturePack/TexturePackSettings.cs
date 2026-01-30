namespace WAM.Core.Builder.TexturePack {
    public readonly struct TexturePackSettings {
        public required uint MaxPackSize { get; init; }
        public required bool AllowMultipleFiles { get; init; }

        public static readonly TexturePackSettings Default = new() {
            AllowMultipleFiles = false,
            MaxPackSize = 1024
        };
    }
}
