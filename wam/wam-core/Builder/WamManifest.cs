using System.Data.Common;
using System.IO;
using System.Text.Json;
using WAM.Core.Builder.JsonTypes.Input;
using WAM.Core.Builder.JsonTypes.Output;
using WAM.Core.Builder.TexturePack;
using WAM.Core.Internal.Generator;

namespace WAM.Core.Builder {
    using InputManifestResult = Result<InputManifest>;

    public sealed class WamManifest:IAssetGenerator {

        private readonly WamManifestSettings settings;
        private readonly TexturePackBuilder texturePackBuilder;

        public WamManifest() {
            settings = WamManifestSettings.Default;
            texturePackBuilder = new(settings.TexturePackSettings);
        }

        public WamManifest(WamManifestSettings settings) {
            this.settings = settings;
            texturePackBuilder = new(settings.TexturePackSettings);
        }

        const string INPUT_MANIFEST_NAME = "manifest.json";
        const string PACK_FILE = "pack.json";

        private readonly SequentialIDGenerator idGenerator = new();
        private readonly UniqueGuidGenerator guidGenerator = new();

        private readonly Dictionary<string,Namespace> namespaces = [];
        private readonly List<FileMap> fileMaps = [];
        private readonly NamespaceBuilder namespaceBuilder = new();

        private readonly List<GeneratedFile> generatedFiles = [];

        private void Reset() {
            namespaces.Clear();
            fileMaps.Clear();
            guidGenerator.Reset();
            idGenerator.Reset();
            namespaceBuilder.Reset();
            texturePackBuilder.Reset();
        }

        private static readonly JsonSerializerOptions jsonOptions = new() {
            DictionaryKeyPolicy = JsonNamingPolicy.KebabCaseLower,
            PropertyNameCaseInsensitive = true,
        };

        public IEnumerable<FileMap> GetFileMaps() {
            return fileMaps;
        }

        public string GetJson() {
            return JsonSerializer.Serialize(namespaces,jsonOptions);
        }

        public IEnumerable<GeneratedFile> GetGeneratedFiles() {
            return generatedFiles;
        }

        private string GetDestinationRoot() {
            // if guid usage is in effect, flatten the namespace folders
            throw new NotImplementedException();
        }

        private (int,string) BindAssetBase(
            string relativePath,
            FileType type
        ) {
            relativePath = settings.UseGuids ? guidGenerator.Next() : relativePath;
            relativePath = namespaceBuilder.QualifyAssetPath(relativePath);

            var ID = idGenerator.Next();
            namespaceBuilder.AddAsset(new() {
                ID = ID,
                Type = type.ToString(),
                Path = relativePath
            });

            var destination = Path.Combine(
                GetDestinationRoot(),
                relativePath
            );
            return (ID,destination);
        }

        public int BindAsset(
            string relativePath,
            string qualifiedPath,
            FileType type
        ) {
            var (ID,destination) = BindAssetBase(
                relativePath,
                type
            );

            var fileMap = new FileMap {
                Source = qualifiedPath,
                Destination = destination
            };
            fileMaps.Add(fileMap);

            return ID;
        }

        public int BindAsset(
            string relativePath,
            FileType type,
            byte[] data
        ) {
            var (ID,destination) = BindAssetBase(
                relativePath,
                type
            );

            var generatedFile = new GeneratedFile {
                Data = data,
                Destination = destination
            };
            generatedFiles.Add(generatedFile);

            return ID;
        }

        private Error? AddNamespace(QualifiedInputManifest manifest) {
            namespaceBuilder.Reset();

            var namespaceDirectories = Directory.GetDirectories(manifest.Path,"*",SearchOption.AllDirectories);

            foreach(var subdirectory in namespaceDirectories) {
                bool useTexturePacking = subdirectory.Contains(PACK_FILE);
                if(subdirectory.Contains(PACK_FILE)) {
                    texturePackBuilder.Reset();
                }
                foreach(var file in Directory.GetFiles(subdirectory)) {
                    if(!FileTypeHelper.TryGetType(file,out var type)) {
                        continue;
                    }
                    BindAsset(file,manifest.Path,type);
                }
                if(useTexturePacking) {
                    var buildResult = texturePackBuilder.Build(subdirectory,this);
                    if(buildResult.IsErr) {
                        return Error.Create($"{buildResult.Error}");
                    }
                    var texturePack = buildResult.Value;
                    foreach(var image in texturePack.Images) {
                        namespaceBuilder.AddImage(image);
                    }
                    foreach(var generatedFile in texturePack.Files) {
                        generatedFiles.Add(generatedFile);
                    }
                }
            }

            return null;
        }

        public Error? Build(string namespaceContentRoot,string targetNamespace) {
            Reset();

            if(string.IsNullOrWhiteSpace(targetNamespace)) {
                return Error.Create(
                    "invalid target namespace"
                );
            }

            if(!Directory.Exists(namespaceContentRoot)) {
                return Error.Create(
                    $"directory '{namespaceContentRoot}' does not exist"
                );
            }

            Dictionary<string,QualifiedInputManifest> allNamespaces = [];

            foreach(var directory in Directory.GetDirectories(namespaceContentRoot,"*",SearchOption.TopDirectoryOnly)) {
                if(!(directory?.Contains(INPUT_MANIFEST_NAME) ?? false)) {
                    continue;
                }
                string path = Path.Join(directory,INPUT_MANIFEST_NAME);
                var result = ScanManifest(path);
                if(result.IsErr) {
                    return Error.Create(result.Error);
                }
                var value = result.Value;
                if(string.IsNullOrWhiteSpace(value.Name)) {
                    return Error.Create($"missing 'namespace' identifier in manifest '{path}'");
                }
                if(value.Includes == null) {
                    return Error.Create($"missing 'includes' in manifest '{path}'");
                }
                if(allNamespaces.ContainsKey(value.Name)) {
                    return Error.Create($"namespace value collision for '{value.Name}'");
                }
                allNamespaces.Add(value.Name,new QualifiedInputManifest() {
                    Name = value.Name,
                    Includes = value.Includes,
                    Path = path
                });
            }

            HashSet<string> requiredNamespaces = [];
            if(allNamespaces.ContainsKey(targetNamespace)) {
                requiredNamespaces.Add(targetNamespace);
            } else {
                return Error.Create($"target namespace '{targetNamespace}' not found");
            }

            foreach(var directory in allNamespaces) {
                var childNamespace = directory.Key;
                foreach(var parentNamespace in directory.Value.Includes) {
                    if(!allNamespaces.ContainsKey(parentNamespace)) {
                        return Error.Create($"namespace '{parentNamespace}' not found, required by '{childNamespace}'");
                    }
                    requiredNamespaces.Add(parentNamespace);
                }
            }

            foreach(var @namespace in requiredNamespaces) {
                var error = AddNamespace(allNamespaces[@namespace]);
                if(error.HasValue) {
                    return Error.Create($"namespace creation failure: {error}");
                }
            }

            return null;
        }

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
                manifest = JsonSerializer.Deserialize<InputManifest>(text,jsonOptions);
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
            return InputManifestResult.Ok(manifest);
        }
    }
}
