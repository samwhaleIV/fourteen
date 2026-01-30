using WAM.Core.Builder.JsonTypes.Output;

namespace WAM.Core.Builder {
    internal sealed class NamespaceBuilder {
        private List<Asset> Assets { get; init; } = [];
        private List<Image> Images { get; init; } = [];
        private List<Json> Json { get; init; } = [];
        private List<Text> Text { get; init; } = [];

        public void AddAsset(Asset asset) => Assets.Add(asset);
        public void AddImage(Image image) => Images.Add(image);
        public void AddJson(Json json) => Json.Add(json);
        public void AddText(Text text) => Text.Add(text);

        public Namespace Build(string name) {
            return new() {
                Name = name,
                Assets = [..Assets],
                Images = [..Images],
                Json = [..Json],
                Text = [..Text]
            };
        }
    }
}
