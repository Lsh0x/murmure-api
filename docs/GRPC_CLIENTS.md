# Génération de clients gRPC pour Murmure

Ce guide explique comment générer des clients gRPC pour Murmure dans différentes langages de programmation.

## Prérequis

1. **Protocole Buffer Compiler (`protoc`)**
   
   Installez le compilateur Protocol Buffers:
   
   **macOS:**
   ```bash
   brew install protobuf
   ```
   
   **Linux (Ubuntu/Debian):**
   ```bash
   sudo apt-get install protobuf-compiler
   ```
   
   **Windows:**
   Téléchargez depuis: https://github.com/protocolbuffers/protobuf/releases

2. **Fichier proto**
   
   Le fichier de définition du service se trouve dans: `proto/murmure.proto`

## Services disponibles

Le service `TranscriptionService` expose deux méthodes:

- **TranscribeFile**: Transcription d'un fichier audio complet (non-streaming)
- **TranscribeStream**: Transcription en streaming bidirectionnel pour la transcription en temps réel

Voir `proto/murmure.proto` pour les détails complets des messages et services.

---

## Python

### Installation

```bash
pip install grpcio grpcio-tools
```

### Génération des stubs

```bash
python -m grpc_tools.protoc \
    -Iproto \
    --python_out=. \
    --grpc_python_out=. \
    proto/murmure.proto
```

Cela génère:
- `murmure_pb2.py` - Messages
- `murmure_pb2_grpc.py` - Services clients

### Exemple d'utilisation

```python
import grpc
from murmure_pb2 import TranscribeFileRequest, TranscribeFileResponse
from murmure_pb2_grpc import TranscriptionServiceStub

# Créer un canal gRPC
channel = grpc.insecure_channel('localhost:50051')
stub = TranscriptionServiceStub(channel)

# Lire le fichier audio
with open('audio.wav', 'rb') as f:
    audio_data = f.read()

# Créer la requête
request = TranscribeFileRequest(
    audio_data=audio_data,
    use_dictionary=True
)

# Appeler le service
try:
    response = stub.TranscribeFile(request)
    if response.success:
        print(f"Transcription: {response.text}")
    else:
        print(f"Erreur: {response.error}")
except grpc.RpcError as e:
    print(f"Erreur gRPC: {e.code()}: {e.details()}")
finally:
    channel.close()
```

### Streaming

```python
def audio_chunks():
    chunk_size = 8192
    with open('audio.wav', 'rb') as f:
        while True:
            chunk = f.read(chunk_size)
            if not chunk:
                break
            yield TranscribeStreamRequest(audio_chunk=chunk)
    # Fin du stream
    yield TranscribeStreamRequest(end_of_stream=True)

responses = stub.TranscribeStream(audio_chunks())
for response in responses:
    if response.is_final:
        if response.final_text:
            print(f"Transcription finale: {response.final_text}")
        elif response.error:
            print(f"Erreur: {response.error}")
    else:
        if response.partial_text:
            print(f"Partiel: {response.partial_text}")
```

**Exemple complet:** Voir `examples/python_client.py`

---

## Node.js / TypeScript

### Installation

```bash
npm install @grpc/grpc-js @grpc/proto-loader
```

Pour TypeScript, ajoutez aussi:
```bash
npm install --save-dev typescript @types/node
```

### Génération des stubs (Option 1: proto-loader)

**JavaScript:**

```javascript
const grpc = require('@grpc/grpc-js');
const protoLoader = require('@grpc/proto-loader');

// Charger le proto
const packageDefinition = protoLoader.loadSync('proto/murmure.proto', {
    keepCase: true,
    longs: String,
    enums: String,
    defaults: true,
    oneofs: true
});

const murmureProto = grpc.loadPackageDefinition(packageDefinition).murmure;
const client = new murmureProto.TranscriptionService(
    'localhost:50051',
    grpc.credentials.createInsecure()
);

// Utilisation
const fs = require('fs');
const audioData = fs.readFileSync('audio.wav');

const request = {
    audio_data: audioData,
    use_dictionary: true
};

client.TranscribeFile(request, (error, response) => {
    if (error) {
        console.error('Erreur:', error);
        return;
    }
    if (response.success) {
        console.log('Transcription:', response.text);
    } else {
        console.error('Erreur:', response.error);
    }
});
```

**TypeScript:**

```typescript
import * as grpc from '@grpc/grpc-js';
import * as protoLoader from '@grpc/proto-loader';
import * as fs from 'fs';

const packageDefinition = protoLoader.loadSync('proto/murmure.proto', {
    keepCase: true,
    longs: String,
    enums: String,
    defaults: true,
    oneofs: true
});

const murmureProto = grpc.loadPackageDefinition(packageDefinition) as any;
const client = new murmureProto.murmure.TranscriptionService(
    'localhost:50051',
    grpc.credentials.createInsecure()
);

const audioData = fs.readFileSync('audio.wav');

client.TranscribeFile(
    { audio_data: audioData, use_dictionary: true },
    (error: grpc.ServiceError | null, response: any) => {
        if (error) {
            console.error('Erreur:', error);
            return;
        }
        if (response.success) {
            console.log('Transcription:', response.text);
        } else {
            console.error('Erreur:', response.error);
        }
    }
);
```

### Génération des stubs (Option 2: ts-proto)

Alternative avec génération de TypeScript typé:

```bash
npm install --save-dev ts-proto
npx protoc \
    --plugin=./node_modules/.bin/protoc-gen-ts_proto \
    --ts_proto_out=. \
    --ts_proto_opt=esModuleInterop=true \
    proto/murmure.proto
```

### Streaming (Node.js)

```javascript
const call = client.TranscribeStream();

call.on('data', (response) => {
    if (response.is_final) {
        if (response.final_text) {
            console.log('Final:', response.final_text);
        }
    } else {
        if (response.partial_text) {
            console.log('Partiel:', response.partial_text);
        }
    }
});

call.on('end', () => {
    console.log('Stream terminé');
});

// Envoyer des chunks
const chunkSize = 8192;
const stream = fs.createReadStream('audio.wav');
stream.on('data', (chunk) => {
    call.write({ audio_chunk: chunk });
});
stream.on('end', () => {
    call.write({ end_of_stream: true });
    call.end();
});
```

---

## Rust

### Installation

Ajoutez à votre `Cargo.toml`:

```toml
[dependencies]
tonic = "0.12"
tokio = { version = "1", features = ["full"] }
prost = "0.12"
```

### Génération des stubs

Créez un fichier `build.rs` à la racine de votre projet:

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(false)  // On génère seulement le client
        .build_client(true)
        .compile_protos(&["proto/murmure.proto"], &["proto"])?;
    Ok(())
}
```

Ajoutez aussi au `Cargo.toml`:

```toml
[build-dependencies]
tonic-build = "0.12"
```

### Exemple d'utilisation

```rust
use tonic::Request;
use murmure::transcription_service_client::TranscriptionServiceClient;
use murmure::{TranscribeFileRequest, TranscribeFileResponse};

mod murmure {
    include!(concat!(env!("OUT_DIR"), "/murmure.rs"));
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = TranscriptionServiceClient::connect(
        "http://localhost:50051"
    ).await?;

    let audio_data = std::fs::read("audio.wav")?;

    let request = Request::new(TranscribeFileRequest {
        audio_data,
        use_dictionary: true,
    });

    let response = client.transcribe_file(request).await?;
    let transcription = response.into_inner();

    if transcription.success {
        println!("Transcription: {}", transcription.text);
    } else {
        eprintln!("Erreur: {}", transcription.error);
    }

    Ok(())
}
```

**Exemples complets:** Voir `examples/rust_file_client.rs`, `examples/rust_streaming_client.rs`

---

## Go

### Installation

```bash
go install google.golang.org/protobuf/cmd/protoc-gen-go@latest
go install google.golang.org/grpc/cmd/protoc-gen-go-grpc@latest
```

### Génération des stubs

```bash
protoc \
    --go_out=. \
    --go_opt=paths=source_relative \
    --go-grpc_out=. \
    --go-grpc_opt=paths=source_relative \
    proto/murmure.proto
```

Cela génère:
- `proto/murmure.pb.go` - Messages
- `proto/murmure_grpc.pb.go` - Services clients

### Exemple d'utilisation

```go
package main

import (
    "context"
    "fmt"
    "io/ioutil"
    "log"
    "time"

    "google.golang.org/grpc"
    "google.golang.org/grpc/credentials/insecure"
    pb "your-module/proto" // Ajustez le chemin
)

func main() {
    conn, err := grpc.Dial(
        "localhost:50051",
        grpc.WithTransportCredentials(insecure.NewCredentials()),
    )
    if err != nil {
        log.Fatalf("Échec de connexion: %v", err)
    }
    defer conn.Close()

    client := pb.NewTranscriptionServiceClient(conn)

    audioData, err := ioutil.ReadFile("audio.wav")
    if err != nil {
        log.Fatalf("Erreur lecture fichier: %v", err)
    }

    ctx, cancel := context.WithTimeout(context.Background(), time.Second*30)
    defer cancel()

    req := &pb.TranscribeFileRequest{
        AudioData:    audioData,
        UseDictionary: true,
    }

    resp, err := client.TranscribeFile(ctx, req)
    if err != nil {
        log.Fatalf("Erreur transcription: %v", err)
    }

    if resp.Success {
        fmt.Printf("Transcription: %s\n", resp.Text)
    } else {
        fmt.Printf("Erreur: %s\n", resp.Error)
    }
}
```

### Streaming

```go
stream, err := client.TranscribeStream(context.Background())
if err != nil {
    log.Fatalf("Erreur création stream: %v", err)
}

// Lire et envoyer des chunks
chunkSize := 8192
audioData, _ := ioutil.ReadFile("audio.wav")

for i := 0; i < len(audioData); i += chunkSize {
    end := i + chunkSize
    if end > len(audioData) {
        end = len(audioData)
    }
    req := &pb.TranscribeStreamRequest{
        RequestType: &pb.TranscribeStreamRequest_AudioChunk{
            AudioChunk: audioData[i:end],
        },
    }
    if err := stream.Send(req); err != nil {
        log.Fatalf("Erreur envoi: %v", err)
    }
}

// Fin du stream
stream.Send(&pb.TranscribeStreamRequest{
    RequestType: &pb.TranscribeStreamRequest_EndOfStream{
        EndOfStream: true,
    },
})

stream.CloseSend()

// Lire les réponses
for {
    resp, err := stream.Recv()
    if err == io.EOF {
        break
    }
    if err != nil {
        log.Fatalf("Erreur réception: %v", err)
    }
    if resp.IsFinal {
        if resp.GetFinalText() != "" {
            fmt.Printf("Final: %s\n", resp.GetFinalText())
        }
    } else {
        if resp.GetPartialText() != "" {
            fmt.Printf("Partiel: %s\n", resp.GetPartialText())
        }
    }
}
```

---

## Java

### Installation

Ajoutez à votre `pom.xml`:

```xml
<dependencies>
    <dependency>
        <groupId>io.grpc</groupId>
        <artifactId>grpc-netty-shaded</artifactId>
        <version>1.58.0</version>
    </dependency>
    <dependency>
        <groupId>io.grpc</groupId>
        <artifactId>grpc-protobuf</artifactId>
        <version>1.58.0</version>
    </dependency>
    <dependency>
        <groupId>io.grpc</groupId>
        <artifactId>grpc-stub</artifactId>
        <version>1.58.0</version>
    </dependency>
</dependencies>

<build>
    <extensions>
        <extension>
            <groupId>kr.motd.maven</groupId>
            <artifactId>os-maven-plugin</artifactId>
            <version>1.7.1</version>
        </extension>
    </extensions>
    <plugins>
        <plugin>
            <groupId>org.xolstice.maven.plugins</groupId>
            <artifactId>protobuf-maven-plugin</artifactId>
            <version>0.6.1</version>
            <configuration>
                <protocArtifact>com.google.protobuf:protoc:3.24.4:exe:${os.detected.classifier}</protocArtifact>
                <pluginId>grpc-java</pluginId>
                <pluginArtifact>io.grpc:protoc-gen-grpc-java:1.58.0:exe:${os.detected.classifier}</pluginArtifact>
            </configuration>
            <executions>
                <execution>
                    <goals>
                        <goal>compile</goal>
                        <goal>compile-custom</goal>
                    </goals>
                </execution>
            </executions>
        </plugin>
    </plugins>
</build>
```

### Génération des stubs

```bash
mvn compile
```

Les stubs seront générés dans `target/generated-sources/protobuf/`.

### Exemple d'utilisation

```java
import io.grpc.ManagedChannel;
import io.grpc.ManagedChannelBuilder;
import io.grpc.StatusRuntimeException;
import murmure.TranscriptionServiceGrpc;
import murmure.TranscribeFileRequest;
import murmure.TranscribeFileResponse;
import java.nio.file.Files;
import java.nio.file.Paths;

public class MurmureClient {
    public static void main(String[] args) throws Exception {
        ManagedChannel channel = ManagedChannelBuilder.forAddress("localhost", 50051)
            .usePlaintext()
            .build();

        TranscriptionServiceGrpc.TranscriptionServiceBlockingStub stub =
            TranscriptionServiceGrpc.newBlockingStub(channel);

        byte[] audioData = Files.readAllBytes(Paths.get("audio.wav"));

        TranscribeFileRequest request = TranscribeFileRequest.newBuilder()
            .setAudioData(com.google.protobuf.ByteString.copyFrom(audioData))
            .setUseDictionary(true)
            .build();

        try {
            TranscribeFileResponse response = stub.transcribeFile(request);
            if (response.getSuccess()) {
                System.out.println("Transcription: " + response.getText());
            } else {
                System.err.println("Erreur: " + response.getError());
            }
        } catch (StatusRuntimeException e) {
            System.err.println("Erreur gRPC: " + e.getStatus());
        } finally {
            channel.shutdown();
        }
    }
}
```

---

## C#

### Installation

Installez les packages NuGet:

```bash
dotnet add package Grpc.Net.Client
dotnet add package Google.Protobuf
dotnet add package Grpc.Tools
```

### Configuration du projet (.csproj)

```xml
<ItemGroup>
    <Protobuf Include="proto/murmure.proto" GrpcServices="Client" />
</ItemGroup>
```

### Exemple d'utilisation

```csharp
using Grpc.Net.Client;
using Murmure;
using Google.Protobuf;

var channel = GrpcChannel.ForAddress("http://localhost:50051");
var client = new TranscriptionService.TranscriptionServiceClient(channel);

var audioData = await File.ReadAllBytesAsync("audio.wav");

var request = new TranscribeFileRequest
{
    AudioData = ByteString.CopyFrom(audioData),
    UseDictionary = true
};

try
{
    var response = await client.TranscribeFileAsync(request);
    if (response.Success)
    {
        Console.WriteLine($"Transcription: {response.Text}");
    }
    else
    {
        Console.WriteLine($"Erreur: {response.Error}");
    }
}
catch (RpcException e)
{
    Console.WriteLine($"Erreur gRPC: {e.Status}");
}
finally
{
    await channel.ShutdownAsync();
}
```

---

## Test avec grpcurl

Pour tester rapidement le service sans écrire de code:

```bash
# Installer grpcurl
# macOS: brew install grpcurl
# Linux: https://github.com/fullstorydev/grpcurl/releases

# Lister les services
grpcurl -plaintext localhost:50051 list

# Voir la définition du service
grpcurl -plaintext localhost:50051 describe murmure.TranscriptionService

# Tester TranscribeFile (nécessite d'encoder l'audio en base64)
echo '{"audio_data":"BASE64_ENCODED_WAV_DATA","use_dictionary":true}' | \
  grpcurl -plaintext -d @ localhost:50051 \
  murmure.TranscriptionService/TranscribeFile
```

---

## Configuration du serveur

Par défaut, le serveur Murmure écoute sur `localhost:50051`. 

Pour changer le port, utilisez la variable d'environnement:
```bash
export MURMURE_GRPC_PORT=50052
```

Voir [docs/SERVER.md](SERVER.md) pour plus de détails sur la configuration du serveur.

---

## Format audio requis

- **Format**: WAV (PCM)
- **Taux d'échantillonnage**: 16 kHz (automatiquement rééchantillonné si différent)
- **Canaux**: Mono (automatiquement converti depuis stéréo)
- **Profondeur de bits**: 16-bit

Le serveur effectue automatiquement les conversions nécessaires si le format diffère.

---

## Dépannage

### Erreur de connexion

- Vérifiez que le serveur Murmure est en cours d'exécution
- Vérifiez que le port est correct (par défaut 50051)
- Vérifiez votre firewall

### Erreurs de génération de stubs

- Assurez-vous que `protoc` est installé et dans votre PATH
- Vérifiez que le chemin vers `proto/murmure.proto` est correct
- Pour certaines langues, des plugins supplémentaires sont nécessaires (voir sections ci-dessus)

### Erreurs de types/propriétés

- Les noms de propriétés peuvent varier selon la langue (snake_case vs camelCase)
- Consultez la documentation de votre bibliothèque gRPC pour les conventions de nommage

---

## Ressources supplémentaires

- [Documentation gRPC officielle](https://grpc.io/docs/)
- [Protocol Buffers Guide](https://developers.google.com/protocol-buffers)
- [Exemples de clients](examples/)
- [Documentation du serveur](SERVER.md)

