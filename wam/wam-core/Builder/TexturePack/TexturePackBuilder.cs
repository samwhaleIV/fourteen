namespace WAM.Core.Builder.TexturePack {
    public sealed class TexturePackBuilder(TexturePackSettings settings) {

        private readonly TexturePackSettings settings = settings;

        public void AddImage(string name) {
            throw new NotImplementedException();
        }
        public Result<TexturePack> Build(string packDirectory,IAssetGenerator assetGenerator) {
            throw new NotImplementedException();
        }

        public void Reset() {
            throw new NotImplementedException();
        }
    }
}
