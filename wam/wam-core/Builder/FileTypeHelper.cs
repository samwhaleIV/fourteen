namespace WAM.Core.Builder {
    public static class FileTypeHelper {
        private static Dictionary<string,FileType> types = new() {
            { "png", FileType.Image },
            { "jpg", FileType.Image },
            { "jpeg", FileType.Image },
            { "json", FileType.JSON },
            { "txt", FileType.Text },
        };
        public static bool TryGetType(string type,out FileType value) {
            return types.TryGetValue(type,out value);
        }
    }
}
