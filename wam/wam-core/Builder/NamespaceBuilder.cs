namespace WAM.Core.Builder {
    internal sealed class NamespaceBuilder {
        private List<HardAsset> HardAssets { get; init; } = [];
        private List<VirtualAsset> VirtualAssets { get; init; } = [];

        private readonly Dictionary<string,int> usedNames = [];

        public void AddHardAsset(HardAsset asset) => HardAssets.Add(asset);
        public void AddVirtualAsset(VirtualAsset image) => VirtualAssets.Add(image);

        public string QualifyAssetName(string name) {
            if(usedNames.TryGetValue(name,out int value)) {
                name = $"{name} - {value}";
                usedNames[name] = value + 1;
            } else {
                usedNames.Add(name,1);
            }
            return name;
        }

        public Namespace Build(string name) {
            return new() {
                Name = name,
                HardAssets = [..HardAssets],
                VirtualAssets = [..VirtualAssets],
            };
        }

        public void Reset() {
            VirtualAssets.Clear();
            HardAssets.Clear();
            usedNames.Clear();
        }
    }
}
