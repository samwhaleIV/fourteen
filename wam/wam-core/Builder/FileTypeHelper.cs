namespace WAM.Core.Builder {
    public static class FileTypeHelper {
        private static readonly Dictionary<string,FileType> types = new() {
            { ".png", FileType.Image },
            { ".jpg", FileType.Image },
            { ".jpeg", FileType.Image },
            { ".json", FileType.Json },
            { ".txt", FileType.Text },
        };
        public static bool TryGetType(string type,out FileType value) {
            return types.TryGetValue(type,out value);
        }
        public static string ToString(this FileType fileType) {
            return fileType switch {
                FileType.Image => "image",
                FileType.Text => "text",
                FileType.Json => "json",
                _ => throw new NotImplementedException(),
            };
        }
    }
}
