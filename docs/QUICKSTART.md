# Quick Start Guide

Get Murmure gRPC Server running in 5 minutes!

## Step 1: Download the Model

```bash
mkdir -p resources
cd resources
curl -L -o /tmp/parakeet-model.zip \
  "https://github.com/Kieirra/murmure-model/releases/download/1.0.0/parakeet-tdt-0.6b-v3-int8.zip"
unzip /tmp/parakeet-model.zip
rm /tmp/parakeet-model.zip
cd ..
```

Verify the model is downloaded:

```bash
ls -la resources/parakeet-tdt-0.6b-v3-int8/
# Should show: encoder-model.int8.onnx, decoder_joint-model.int8.onnx, nemo128.onnx, vocab.txt
```

## Step 2: Set Environment Variables

Create a `.env` file (or export directly):

```bash
cp .env.example .env
```

Or set directly:

```bash
export MURMURE_MODEL_PATH=./resources/parakeet-tdt-0.6b-v3-int8
export MURMURE_CC_RULES_PATH=./resources/cc-rules
export MURMURE_GRPC_PORT=50051
export MURMURE_LOG_LEVEL=info
```

## Step 3: Build the Server

```bash
cd src-tauri
cargo build --release --bin murmure-server
```

This will create `target/release/murmure-server`.

## Step 4: Run the Server

```bash
# From src-tauri directory
export MURMURE_MODEL_PATH=../resources/parakeet-tdt-0.6b-v3-int8
export MURMURE_CC_RULES_PATH=../resources/cc-rules
./target/release/murmure-server
```

You should see:
```
Starting Murmure gRPC Server...
Configuration loaded: gRPC port = 50051
Model initialized
gRPC server listening on 0.0.0.0:50051
```

## Step 5: Test with a Client

### Python Client Example

```bash
cd examples
pip install grpcio grpcio-tools
python -m grpc_tools.protoc -I../proto --python_out=. --grpc_python_out=. ../proto/murmure.proto
python python_client.py path/to/audio.wav
```

### Using grpcurl

```bash
# Install grpcurl: brew install grpcurl (macOS) or download from GitHub
# Test connection
grpcurl -plaintext localhost:50051 list
```

## Troubleshooting

### Model Not Found
- Verify `MURMURE_MODEL_PATH` points to the correct directory
- Check that all .onnx files are present

### CC Rules Not Found
- Verify `MURMURE_CC_RULES_PATH` points to `resources/cc-rules`
- Dictionary corrections will be disabled if not found (non-critical)

### Port Already in Use
- Change `MURMURE_GRPC_PORT` to another port (e.g., 50052)
- Or stop the process using port 50051

## Next Steps

- Read [README_SERVER.md](../README_SERVER.md) for detailed documentation
- Check [examples/](../examples/) for client implementations
- See [SERVER.md](./SERVER.md) for API documentation

