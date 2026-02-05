namespace WAM.Core.Builder {
    internal sealed class NamespaceBuilder {
        private List<HardAsset> HardAssets { get; init; } = [];
        private List<VirtualAsset> VirtualAssets { get; init; } = [];
        private List<VirtualImageAsset> VirtualImageAssets { get; init; } = [];
        private List<VirtualModelAsset> VirtualModelAssets { get; init; } = [];

        private readonly Dictionary<string,int> usedNames = [];

        public void AddHardAsset(HardAsset hardAsset) {
            HardAssets.Add(hardAsset);
        }

        public void AddVirtualAsset(VirtualAsset virtualAsset) {
            VirtualAssets.Add(virtualAsset);
        }

        public void AddVirtualImageAsset(VirtualImageAsset virtualImageAsset) {
            VirtualImageAssets.Add(virtualImageAsset);
        }

        public void AddVirtualModelAsset(VirtualModelAsset virtualModelAsset) {
            VirtualModelAssets.Add(virtualModelAsset);
        }

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
                VirtualImageAssets = [..VirtualImageAssets],
                VirtualAssets = [..VirtualAssets],
                VirtualModelAssets = [..VirtualModelAssets],
            };
        }

        public void Reset() {
            VirtualAssets.Clear();
            HardAssets.Clear();
            usedNames.Clear();
            VirtualModelAssets.Clear();
        }
    }
}
