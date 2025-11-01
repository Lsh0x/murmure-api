#!/usr/bin/env python3
"""
Example Python client for Murmure gRPC Server

Requirements:
    pip install grpcio grpcio-tools

Generate Python stubs:
    python -m grpc_tools.protoc -I../proto --python_out=. --grpc_python_out=. ../proto/murmure.proto
"""

import grpc
import sys
from pathlib import Path

# Import generated stubs (after running protoc)
# from murmure_pb2 import TranscribeFileRequest, TranscribeFileResponse
# from murmure_pb2_grpc import TranscriptionServiceStub


def transcribe_file(audio_path: str, server_address: str = "localhost:50051"):
    """Transcribe an audio file using the TranscribeFile RPC."""
    # Read audio file
    with open(audio_path, "rb") as f:
        audio_data = f.read()

    # Create gRPC channel and stub
    channel = grpc.insecure_channel(server_address)
    stub = TranscriptionServiceStub(channel)

    # Create request
    request = TranscribeFileRequest(
        audio_data=audio_data,
        use_dictionary=True
    )

    # Call RPC
    try:
        response = stub.TranscribeFile(request)
        if response.success:
            print(f"Transcription: {response.text}")
            return response.text
        else:
            print(f"Error: {response.error}")
            return None
    except grpc.RpcError as e:
        print(f"gRPC error: {e.code()}: {e.details()}")
        return None
    finally:
        channel.close()


def transcribe_stream(audio_path: str, server_address: str = "localhost:50051"):
    """Transcribe an audio file using streaming RPC."""
    channel = grpc.insecure_channel(server_address)
    stub = TranscriptionServiceStub(channel)

    # Read audio file in chunks
    chunk_size = 8192
    with open(audio_path, "rb") as f:
        def audio_chunks():
            while True:
                chunk = f.read(chunk_size)
                if not chunk:
                    break
                yield TranscribeStreamRequest(
                    request_type=TranscribeStreamRequest.RequestType.AudioChunk(chunk)
                )
            # Send end of stream
            yield TranscribeStreamRequest(
                request_type=TranscribeStreamRequest.RequestType.EndOfStream(True)
            )

    try:
        responses = stub.TranscribeStream(audio_chunks())
        for response in responses:
            if response.is_final:
                if response.response_type == TranscribeStreamResponse.ResponseType.FinalText:
                    print(f"Final transcription: {response.final_text}")
                    return response.final_text
                elif response.response_type == TranscribeStreamResponse.ResponseType.Error:
                    print(f"Error: {response.error}")
                    return None
            else:
                if response.response_type == TranscribeStreamResponse.ResponseType.PartialText:
                    print(f"Partial: {response.partial_text}")
    except grpc.RpcError as e:
        print(f"gRPC error: {e.code()}: {e.details()}")
        return None
    finally:
        channel.close()


if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python python_client.py <audio_file.wav> [server_address]")
        sys.exit(1)

    audio_file = sys.argv[1]
    server = sys.argv[2] if len(sys.argv) > 2 else "localhost:50051"

    print(f"Transcribing {audio_file} using server at {server}")
    
    # Try streaming first, fall back to file-based
    result = transcribe_stream(audio_file, server)
    if result is None:
        print("Streaming failed, trying file-based transcription...")
        result = transcribe_file(audio_file, server)

