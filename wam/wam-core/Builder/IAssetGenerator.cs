namespace WAM.Core.Builder {
    public interface IAssetGenerator {
        public int BindAsset(
            string relativePath,
            string qualifiedPath,
            FileType type
        );
        public int BindAsset(
            string relativePath,
            FileType type,
            byte[] data
        );
    }
}
