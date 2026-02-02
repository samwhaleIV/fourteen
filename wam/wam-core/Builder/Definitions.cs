
using System.Text.Json;
using System.Text.Json.Serialization;

namespace WAM.Core.Builder {

    public readonly record struct FileMap(
        string Source,
        string Destination
    );

    public readonly record struct GeneratedFile(
        string Destination,
        byte[] Data
    );

    public readonly record struct Area(
        int X,
        int Y,
        int Width,
        int Height
    );

    public readonly record struct VirtualAsset(
        int ID,
        FileType Type,
        [property: JsonConverter(typeof(ForwardSlashConverter))]
        string Name
    );

    public readonly record struct VirtualImageAsset(
        int ID,
        [property: JsonConverter(typeof(ForwardSlashConverter))]
        string Name,
        Area Area
    );

    public readonly record struct HardAsset(
        int ID,
        FileType Type,
        [property: JsonConverter(typeof(ForwardSlashConverter))]
        string Source
    );

    public readonly record struct Namespace(
        HardAsset[] HardAssets,
        VirtualAsset[] VirtualAssets,
        VirtualImageAsset[] VirtualImageAssets,
        [property: JsonIgnore] string Name
    );

    public enum FileType {
        Image,
        Text,
        Json
    };

    public static class FileTypeHelper {
        private static readonly Dictionary<string,FileType> inputTypes = new() {
            { ".png", FileType.Image },
            { ".jpg", FileType.Image },
            { ".jpeg", FileType.Image },
            { ".json", FileType.Json },
            { ".txt", FileType.Text },
        };
        public static bool TryGetType(string type,out FileType value) {
            return inputTypes.TryGetValue(type,out value);
        }
    }

    public sealed class ForwardSlashConverter:JsonConverter<string> {
        public override string? Read(ref Utf8JsonReader reader,Type typeToConvert,JsonSerializerOptions options) {
            return reader.GetString();
        }
        public override void Write(Utf8JsonWriter writer,string value,JsonSerializerOptions options) {
            writer.WriteStringValue(value.Replace('\\','/'));
        }
    }

    public sealed class FileTypeConverter:JsonConverter<FileType> {
        public override FileType Read(ref Utf8JsonReader reader,Type typeToConvert,JsonSerializerOptions options) {
            var value = reader.GetString();
            if(string.IsNullOrWhiteSpace(value)) {
                throw new JsonException("No value from reader");
            }
            return value?.ToLowerInvariant() switch {
                "image" => FileType.Image,
                "text" => FileType.Text,
                "json" => FileType.Json,
                _ => throw new JsonException($"Unknown file type: {value}")
            };
        }

        public override void Write(Utf8JsonWriter writer,FileType value,JsonSerializerOptions options) {
            writer.WriteStringValue(value.ToString().ToLowerInvariant());
        }
    }
}
