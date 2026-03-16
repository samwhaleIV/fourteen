
using System.Text.Json;
using System.Text.Json.Serialization;

namespace WAM.Core.Builder {

    public sealed class InputManifest {
        public string? Name { get; set; }
        public string[]? Includes { get; set; }
    }

    public sealed class ModelManifestMeshletDescriptor {
        public string? Diffuse { get; set; }
        public string? Lightmap { get; set; }
    }

    public sealed class ModelManifest {
        public string? Model { get; set; }
        public ModelManifestMeshletDescriptor[]? Meshlets { get; set; }
    }

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
        uint ID,
        [property: JsonConverter(typeof(ForwardSlashConverter))]
        string Name
    );

    public readonly record struct VirtualImageSliceAsset(
        uint ID,
        [property: JsonConverter(typeof(ForwardSlashConverter))]
        string Name,
        Area Slice
    );

    public readonly record struct VirtualModelAssetMeshletDescriptor(
        uint? Diffuse,
        uint? Lightmap
    );

    public readonly record struct VirtualModelAsset(
        uint? ID,
        [property: JsonConverter(typeof(ForwardSlashConverter))]
        string Name,
        VirtualModelAssetMeshletDescriptor[] Meshlets
        //TODO: string? CollisionName
    );

    public readonly record struct HardAsset(
        uint ID,
        FileType Type,
        [property: JsonConverter(typeof(ForwardSlashConverter))]
        string Source
    );

    public readonly record struct ImageSizeHint(
        uint ID,
        uint X,
        uint Y
    );

    public readonly record struct Namespace(
        HardAsset[] HardAssets,
        VirtualAsset[] VirtualAssets,
        VirtualImageSliceAsset[] VirtualImageSliceAssets,
        VirtualModelAsset[] VirtualModelAssets,
        ImageSizeHint[] ImageSizeHints,
        [property: JsonIgnore] string Name
    );

    public enum FileType {
        Image,
        Text,
        Model
    };

    public static class FileTypeHelper {
        private static readonly Dictionary<string,FileType> inputTypes = new() {
            { ".png", FileType.Image },
            { ".jpg", FileType.Image },
            { ".jpeg", FileType.Image },
            { ".txt", FileType.Text },
            { ".glb", FileType.Model },
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
                "model" => FileType.Model,
                _ => throw new JsonException($"Unknown file type: {value}")
            };
        }

        public override void Write(Utf8JsonWriter writer,FileType value,JsonSerializerOptions options) {
            writer.WriteStringValue(value.ToString().ToLowerInvariant());
        }
    }
}
