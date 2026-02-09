using System.Text.Json;
using WAM.Core.Builder.TexturePack;
using WAM.Core.Internal.Generator;

namespace WAM.Core.Builder {
    using InputManifestResult = Result<InputManifest>;
    using ModelManifestResult = Result<ModelManifest>;

    public sealed class WamManifest {

        private readonly JsonSerializerOptions jsonOptions;
        private readonly TexturePackBuilder texturePackBuilder;
        private readonly WamManifestSettings settings;

        public WamManifest(WamManifestSettings settings) {
            jsonOptions = new() {
                DictionaryKeyPolicy = JsonNamingPolicy.KebabCaseLower,
                PropertyNamingPolicy = JsonNamingPolicy.KebabCaseLower,
                PropertyNameCaseInsensitive = true,
                WriteIndented = !settings.CompressManifest,
                IndentSize = JSON_INDENT_SIZE,
                IndentCharacter = JSON_INDENT_CHAR,
            };
            jsonOptions.Converters.Add(new FileTypeConverter());

            var texturePackSettings = settings.TexturePackSettings ?? new TexturePackSettings();
            texturePackBuilder = new(texturePackSettings);
            this.settings = settings;
        }

        const string INPUT_MANIFEST_NAME = "manifest.json";
        const string PACK_FILE = "pack";
        const int JSON_INDENT_SIZE = 4;
        const char JSON_INDENT_CHAR = ' ';

        const string MODEL_MANIFEST_NAME = "model.json";

        private readonly SequentialIDGenerator idGenerator = new();
        private readonly UniqueGuidGenerator guidGenerator = new();

        private readonly Dictionary<string,Namespace> namespaces = [];
        private readonly List<FileMap> fileMaps = [];
        private readonly NamespaceBuilder namespaceBuilder = new();

        private readonly List<GeneratedFile> generatedFiles = [];
        private readonly Dictionary<uint,string> compileTimeDestinations = [];

        private void Reset() {
            namespaces.Clear();
            fileMaps.Clear();
            guidGenerator.Reset();
            idGenerator.Reset();
            namespaceBuilder.Reset();
            texturePackBuilder.Reset();
            compileTimeDestinations.Clear();
        }

        public IEnumerable<FileMap> GetFileMaps() {
            return fileMaps;
        }

        public string GetJson() {
            return JsonSerializer.Serialize(namespaces,jsonOptions);
        }

        public IEnumerable<GeneratedFile> GetGeneratedFiles() {
            return generatedFiles;
        }

        public string GetAssetDestination(uint assetID) {
            return compileTimeDestinations[assetID];
        }

        public uint BindAsset(
            string runtimeFileName,
            string runtimeNamespace,
            string compileTimeSourcePath,
            string fileTypeExtension,
            FileType type
        ) {
            runtimeFileName = namespaceBuilder.QualifyAssetName(
                settings.UseGuids ? guidGenerator.Next() : Path.Combine(runtimeNamespace,runtimeFileName)
            );
            var ID = idGenerator.Next();

            namespaceBuilder.AddHardAsset(new() {
                ID = ID,
                Type = type,
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

            compileTimeDestinations.Add(ID,compileTimeDestination);

            if(!string.IsNullOrWhiteSpace(compileTimeSourcePath)) {
                var fileMap = new FileMap {
                    Source = compileTimeSourcePath,
                    Destination = compileTimeDestination
                };
                fileMaps.Add(fileMap);
            }

            return ID;
        }

        private (Error? Error,uint? ID) TryGetModelItem(
            QualifiedInputManifest manifest,
            string directory,
            string runtimeFileName,
            string? item,
            string itemKey,
            FileType requiredType
        ) {
            uint? assetID = null;
            if(!string.IsNullOrWhiteSpace(item)) {
                var itemPath = Path.Combine(directory,item);
                if(!File.Exists(itemPath)) {
                    return (Error.Create($"model manifest '{runtimeFileName}' points to item '{item}' but it does not exist"), null);
                }
                if(!FileTypeHelper.TryGetType(Path.GetExtension(itemPath),out var type) || type != requiredType) {
                    return (Error.Create($"model manifest '{runtimeFileName}' points to item '{item}' but it is not of expected type '{requiredType}'"), null);
                }
                return (null, BindAsset(
                    Path.Combine(runtimeFileName,itemKey),
                    //$"{runtimeFileName}.{itemKey}",
                    manifest.Name,
                    itemPath,
                    Path.GetExtension(itemPath),
                    requiredType
                ));
            }
            return (null, assetID);
        }

        private Error? BuildModel(
            QualifiedInputManifest manifest,
            string directory
        ) {
            var runtimeFileName = Path.GetRelativePath(manifest.Path,directory);
            var result = ScanModelManifest(Path.Combine(directory,MODEL_MANIFEST_NAME));
            if(result.IsErr) {
                return Error.Create(result.Error);
            }
            var modelManifest = result.Value;

            var model = TryGetModelItem(manifest,directory,runtimeFileName,modelManifest.Model,"model",FileType.Model);
            if(model.Error != null) {
                return model.Error;
            }

            var diffuse = TryGetModelItem(manifest,directory,runtimeFileName,modelManifest.Diffuse,"diffuse",FileType.Image);
            if(diffuse.Error != null) {
                return diffuse.Error;
            }

            var lightmap = TryGetModelItem(manifest,directory,runtimeFileName,modelManifest.Lightmap,"lightmap",FileType.Image);
            if(lightmap.Error != null) {
                return lightmap.Error;
            }

            namespaceBuilder.AddVirtualModelAsset(new() {
                Name = runtimeFileName,
                ModelID = model.ID,
                DiffuseID = diffuse.ID,
                LightmapID = lightmap.ID
            });

            return null;
        }

        private Error? BuildPack(QualifiedInputManifest manifest,string directory) {
            texturePackBuilder.Reset();
            var files = Directory.GetFiles(directory,"*",SearchOption.TopDirectoryOnly);
            foreach(var file in files) {
                if(directory == manifest.Path && Path.GetFileName(file) == INPUT_MANIFEST_NAME) {
                    continue;
                }
                if(!FileTypeHelper.TryGetType(Path.GetExtension(file),out var type) || type != FileType.Image) {
                    continue;
                }
                texturePackBuilder.AddImage(file);
            }

            var runtimeFileName = Path.GetRelativePath(manifest.Path,directory);
            var buildResult = texturePackBuilder.Build(runtimeFileName,manifest.Name,this);
            if(buildResult.IsErr) {
                return Error.Create($"{buildResult.Error}");
            }
            var texturePack = buildResult.Value;
            foreach(var image in texturePack.Images) {
                namespaceBuilder.AddVirtualImageAsset(image);
            }
            foreach(var generatedFile in texturePack.Files) {
                generatedFiles.Add(generatedFile);
            }
            
            return null;
        }

        private Error? BuildAnyFiles(QualifiedInputManifest manifest,string directory) {
            var files = Directory.GetFiles(directory,"*",SearchOption.TopDirectoryOnly);
            foreach(var file in files) {
                if(directory == manifest.Path && Path.GetFileName(file) == INPUT_MANIFEST_NAME) {
                    continue;
                }
                if(!FileTypeHelper.TryGetType(Path.GetExtension(file),out var type)) {
                    continue;
                }

                var runtimeFileName = Path.ChangeExtension(
                    Path.GetRelativePath(manifest.Path,file),
                    null
                );

                bool shouldReformatImage = type == FileType.Image && false; // TODO: Bind Asset, Create virtual image asset, Reformat Images

                if(shouldReformatImage) {
                    throw new NotImplementedException();
                } else {
                    var id = BindAsset(
                        runtimeFileName,
                        manifest.Name,
                        file,
                        Path.GetExtension(file),
                        type
                    );
                    namespaceBuilder.AddVirtualAsset(new() {
                        Name = runtimeFileName,
                        ID = id
                    });
                }
            }
            return null;
        }

        private Error? WalkDirectory(QualifiedInputManifest manifest,string directory) {

            /* The namespace root is the most special folder of all. It can't be anything else, so we don't check for other special cases. */
            if(directory != manifest.Path) {

                /* Special folder modes - these do not recurse for generic assets. Subfolders are controller by the special folder modes. */
                bool buildPack = File.Exists(Path.Combine(directory,PACK_FILE));
                bool buildModel = File.Exists(Path.Combine(directory,MODEL_MANIFEST_NAME));

                if(buildPack && buildModel) {
                    return Error.Create("conflicting special directory type; can't be a model and a texture pack");
                }

                if(buildModel) {
                    var error = BuildModel(manifest,directory);
                    if(error.HasValue) {
                        return error;
                    }
                    return null;
                }

                if(buildPack) {
                    var error = BuildPack(manifest,directory);
                    if(error.HasValue) {
                        return error;
                    }
                    return null;
                }
            }

            var filesResult = BuildAnyFiles(manifest,directory);
            if(filesResult != null) {
                return filesResult;
            }

            var subdirectories = Directory.GetDirectories(directory,"*",SearchOption.TopDirectoryOnly);

            foreach(var subdirectory in subdirectories) {
                var directoryResult = WalkDirectory(manifest,subdirectory);
                if(directoryResult != null) {
                    return directoryResult;
                }
            }
            return null;
        }

        private Error? AddNamespace(QualifiedInputManifest manifest) {
            namespaceBuilder.Reset();
            var result = WalkDirectory(manifest,manifest.Path);
            if(result.HasValue) {
                return result;
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
                    Console.WriteLine($"is '{directory}' a namespace? it does not include a manifest");
                    continue;
                }
                var result = ScanManifest(manifestPath);
                if(result.IsErr) {
                    Console.WriteLine($"is '{directory}' a namespace? its manifest is bad: {result.Error}");
                    continue;
                }
                var value = result.Value;
                if(string.IsNullOrWhiteSpace(value.Name)) {
                    Console.WriteLine($"is '{directory}' a namespace? its manifest does not include a namespace identifier");
                    continue;
                }
                if(allNamespaces.ContainsKey(value.Name)) {
                    return Error.Create($"namespace value collision for '{value.Name}'");
                }
                allNamespaces.Add(value.Name,new QualifiedInputManifest() {
                    Name = value.Name,
                    Includes = value.Includes ?? [],
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
                    return Error.Create($"namespace creation failure: {error.Value.Message}");
                }
            }

            return null;
        }

        private InputManifestResult ScanManifest(string manifestPath) {
            string text;
            try {
                text = File.ReadAllText(manifestPath);
            } catch(Exception exception) {
                return InputManifestResult.Err(
                    $"could not read manifest file '{manifestPath}': {exception.Message}"
                );
            }
            InputManifest? manifest;
            try {
                manifest = JsonSerializer.Deserialize<InputManifest>(text,jsonOptions);
            } catch(Exception exception) {
                return InputManifestResult.Err(
                    $"invalid manifest file '{manifestPath}': {exception.Message}"
                );
            }
            if(manifest == null) {
                return InputManifestResult.Err(
                    $"manifest decode failure '{manifestPath}'"
                );
            }
            return InputManifestResult.Ok(manifest);
        }

        private ModelManifestResult ScanModelManifest(string manifestPath) {
            string text;
            try {
                text = File.ReadAllText(manifestPath);
            } catch(Exception exception) {
                return ModelManifestResult.Err(
                    $"could not read model manifest file '{manifestPath}': {exception.Message}"
                );
            }
            ModelManifest? manifest;
            try {
                manifest = JsonSerializer.Deserialize<ModelManifest>(text,jsonOptions);
            } catch(Exception exception) {
                return ModelManifestResult.Err(
                    $"invalid model manifest file '{manifestPath}': {exception.Message}"
                );
            }
            if(manifest == null) {
                return ModelManifestResult.Err(
                    $"model manifest decode failure '{manifestPath}'"
                );
            }
            return ModelManifestResult.Ok(manifest);
        }
    }
}
