using System.Text.Json;
using WAM.Core.Builder.TexturePack;
using WAM.Core.Internal.Generator;

namespace WAM.Core.Builder {
    using InputManifestResult = Result<InputManifest>;

    public sealed class WamManifest(WamManifestSettings settings) {

        private readonly TexturePackBuilder texturePackBuilder = new(settings.TexturePackSettings);

        const string INPUT_MANIFEST_NAME = "manifest.json";
        const string PACK_FILE = "pack";

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

        public int BindAsset(
            string runtimeFileName,
            string runtimeNamespace,
            string compileTimeSourcePath,
            string fileTypeExtension,
            FileType type,
            byte[]? data = null
        ) {
            runtimeFileName = namespaceBuilder.QualifyAssetName(
                settings.UseGuids ? guidGenerator.Next() : Path.Combine(runtimeNamespace,runtimeFileName)
            );
            var ID = idGenerator.Next();

            namespaceBuilder.AddHardAsset(new() {
                ID = ID,
                Type = type.ToString(),
                Source = $"{runtimeFileName}{fileTypeExtension}"
            });

            var compileTimeDestination = Path.Combine(
                settings.Destination,
                runtimeFileName
            );

            compileTimeDestination = Path.ChangeExtension(
                compileTimeDestination,
                fileTypeExtension
            );

            if(data != null) {
                var generatedFile = new GeneratedFile {
                    Data = data,
                    Destination = compileTimeDestination
                };
                generatedFiles.Add(generatedFile);
            } else {
                var fileMap = new FileMap {
                    Source = compileTimeSourcePath,
                    Destination = compileTimeDestination
                };
                fileMaps.Add(fileMap);
            }

            return ID;
        }

        private Error? AddNamespace(QualifiedInputManifest manifest) {
            namespaceBuilder.Reset();

            string[] namespaceDirectories = [
                manifest.Path,
                ..Directory.GetDirectories(manifest.Path,"*",SearchOption.AllDirectories)
            ];

            foreach(var subdirectory in namespaceDirectories) {
                bool useTexturePacking = File.Exists(Path.Combine(subdirectory,PACK_FILE));
                if(useTexturePacking) {
                    texturePackBuilder.Reset();
                }
                var subdirectoryFiles = Directory.GetFiles(subdirectory,"*",SearchOption.TopDirectoryOnly);
                foreach(var file in subdirectoryFiles) {
                    if(subdirectory == manifest.Path && Path.GetFileName(file) == INPUT_MANIFEST_NAME) {
                        continue;
                    }
                    if(!FileTypeHelper.TryGetType(Path.GetExtension(file),out var type)) {
                        continue;
                    }

                    if(type == FileType.Image && useTexturePacking) {
                        texturePackBuilder.AddImage(file);
                        continue;
                    }

                    var runtimeFileName = Path.ChangeExtension(
                        Path.GetRelativePath(manifest.Path,file),
                        null
                    );

                    var id = BindAsset(
                        runtimeFileName,
                        manifest.Name,
                        file,
                        Path.GetExtension(file),
                        type
                    );

                    namespaceBuilder.AddVirtualAsset(new() {
                        Type = type.ToString(),
                        Name = runtimeFileName,
                        ID = id
                    });
                }
                if(useTexturePacking) {
                    var runtimeFileName = Path.GetRelativePath(manifest.Path,subdirectory);
                    var buildResult = texturePackBuilder.Build(runtimeFileName,manifest.Name,this);
                    if(buildResult.IsErr) {
                        return Error.Create($"{buildResult.Error}");
                    }
                    var texturePack = buildResult.Value;
                    foreach(var image in texturePack.Images) {
                        namespaceBuilder.AddVirtualAsset(image);
                    }
                    foreach(var generatedFile in texturePack.Files) {
                        generatedFiles.Add(generatedFile);
                    }
                }
            }

            namespaces[manifest.Name] = namespaceBuilder.Build(manifest.Name);

            return null;
        }

        public Error? Build() {
            Reset();

            var namespaceRoot = settings.Source;

            if(string.IsNullOrWhiteSpace(settings.TargetNamespace)) {
                return Error.Create(
                    "invalid target namespace"
                );
            }

            if(!Directory.Exists(namespaceRoot)) {
                return Error.Create(
                    $"directory '{namespaceRoot}' does not exist"
                );
            }

            Dictionary<string,QualifiedInputManifest> allNamespaces = [];

            foreach(var directory in Directory.GetDirectories(namespaceRoot,"*",SearchOption.TopDirectoryOnly)) {
                if(string.IsNullOrWhiteSpace(directory)) {
                    continue;
                }
                var manifestPath = Path.Combine(directory,INPUT_MANIFEST_NAME);
                if(!File.Exists(manifestPath)) {
                    continue;
                }
                var result = ScanManifest(manifestPath);
                if(result.IsErr) {
                    return Error.Create(result.Error);
                }
                var value = result.Value;
                if(string.IsNullOrWhiteSpace(value.Name)) {
                    return Error.Create($"missing 'namespace' identifier in manifest '{manifestPath}'");
                }
                if(value.Includes == null) {
                    return Error.Create($"missing 'includes' in manifest '{manifestPath}'");
                }
                if(allNamespaces.ContainsKey(value.Name)) {
                    return Error.Create($"namespace value collision for '{value.Name}'");
                }
                allNamespaces.Add(value.Name,new QualifiedInputManifest() {
                    Name = value.Name,
                    Includes = value.Includes,
                    Path = directory
                });
            }

            HashSet<string> requiredNamespaces = [];

            var targetNamespace = settings.TargetNamespace;

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

            //todo: Write a manifest, write the files, copy the files
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
