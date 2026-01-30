using System;
using System.Diagnostics;
using System.Text.Json;
using System.Text.Json.Serialization;
using System.Xml.Linq;

#pragma warning disable CS8618

namespace WAM.Core {

    using InputManifestResult = Result<InputManifest>;
    using ManifestResult = Result<WamManifest>;

    public sealed class WamManifest {
        const string INPUT_MANIFEST_NAME = "manifest.json";
        const string OUTPUT_MANIFEST_NAME = "manifest.json";

        public Namespace[] namespaces { get; set; }

        private static InputManifestResult ScanManifest(string path) {
            string text;
            try {
                text = File.ReadAllText(path);
            } catch(Exception exception) {
                return InputManifestResult.Err(
                    $"could not read manifest file '{path}': {exception.Message}"
                );
            }
            InputManifest? manifest;
            try {
                manifest = JsonSerializer.Deserialize<InputManifest>(text);
            } catch(Exception exception) {
                return InputManifestResult.Err(
                    $"invalid manifest file '{path}': {exception.Message}"
                    );
            }
            if(manifest == null) {
                return InputManifestResult.Err(
                    $"manifest decode failure '{path}'"
                );
            }
            if(string.IsNullOrWhiteSpace(manifest.name)) {
                return InputManifestResult.Err(
                    $"manifest '{path}' does not contain a 'name' value"
                );
            }
            manifest.path = path;
            return InputManifestResult.Ok(manifest);
        }

        public static ManifestResult Create(string namespaceContentRoot,string targetNamespace) {
            if(string.IsNullOrWhiteSpace(targetNamespace)) {
                return ManifestResult.Err(
                    "invalid target namespace"
                );
            }

            if(!Directory.Exists(namespaceContentRoot)) {
                return ManifestResult.Err(
                    "directory does not exist"
                );
            }

            Dictionary<string,InputManifest> namespaceDirectories = new();

            foreach(var directory in Directory.GetDirectories(namespaceContentRoot,"*",SearchOption.TopDirectoryOnly)) {
                if(directory?.Contains(INPUT_MANIFEST_NAME) ?? false) {
                    string path = Path.Join(directory,INPUT_MANIFEST_NAME);
                    var result = ScanManifest(path);
                    if(result.IsErr) {
                        return ManifestResult.Err(result.Error);
                    }
                    namespaceDirectories.Add(result.Value.name,result.Value);
                }
            }

            if(!namespaceDirectories.ContainsKey(targetNamespace)) {
                return ManifestResult.Err("target namespace not found");
            }

            foreach(var directory in namespaceDirectories) {
                var childNamespace = directory.Key;

                if(directory.Value.includes == null) {
                    continue;
                }
                foreach(var parentNamespace in directory.Value.includes) {
                    if(!namespaceDirectories.ContainsKey(parentNamespace)) {
                        return ManifestResult.Err($"namespace '{parentNamespace}' not found, required by '{childNamespace}'");
                    }
                }
            }

            throw new NotImplementedException();
            
        }

        public string GetJSON() {
            return JsonSerializer.Serialize(this);
        }
    }

    public sealed class InputManifest {
        public string name { get; set; }
        public string[] includes { get; set; }
        [JsonIgnore]
        public string path { get; set; }
    }

    public sealed class Namespace {
        public string name { get; set;  }
        public Asset[] assets { get; set; }
        public Image[] images { get; set; }
        public Text[] text { get; set; }
    }

    public sealed class Asset {
        public string type { get; set; }
        public string path { get; set; }
        public int id { get; set; }
    }

    public sealed class  Image {
        public string name { get; set; }
        public int id { get; set; }
        public Area area { get; set; }
    }

    public sealed class Area {
        public int x { get; set; }
        public int y { get; set; }
        public int width { get; set; }
        public int height { get; set; }
    }

    public sealed class JSON {
        public string name { get; set; }
        public int id { get; set; }
    }
    public sealed class Text {
        public string name { get; set; }
        public int id { get; set; }
    }
}

#pragma warning restore CS8618
