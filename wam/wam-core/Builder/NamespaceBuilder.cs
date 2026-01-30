using WAM.Core.Builder.JsonTypes.Output;

namespace WAM.Core.Builder {
    internal sealed class NamespaceBuilder {
        private List<Asset> Assets { get; init; } = [];
        private List<Image> Images { get; init; } = [];
        private List<Json> Json { get; init; } = [];
        private List<Text> Text { get; init; } = [];

        private HashSet<string> usedAssets = [];

        public void AddAsset(Asset asset) => Assets.Add(asset);
        public void AddImage(Image image) => Images.Add(image);
        public void AddJson(Json json) => Json.Add(json);
        public void AddText(Text text) => Text.Add(text);

        public string QualifyAssetPath(string assetPath) {
            throw new NotImplementedException(); //if assetpath is alerady in usedassets, return the asset with the next available number
        }

        public Namespace Build(string name) {
            return new() {
                Name = name,
                Assets = [..Assets],
                Images = [..Images],
                Json = [..Json],
                Text = [..Text]
            };
        }

        public void Reset() {
            Assets.Clear();
            Images.Clear();
            Json.Clear();
            Text.Clear();
            usedAssets.Clear();
        }
    }
}
