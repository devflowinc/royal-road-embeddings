# Load model directly
from typing import List
from transformers import AutoTokenizer, AutoModel
import uvicorn
import torch
from fastapi import FastAPI
from fastapi.responses import JSONResponse
from pydantic import BaseModel

# Create a Flask app
app = FastAPI()

if torch.cuda.is_available():
    # Initialize CUDA device
    device = torch.device("cuda")
else:
    device = torch.device("cpu")

tokenizer = AutoTokenizer.from_pretrained("BAAI/bge-large-en")
model = AutoModel.from_pretrained("BAAI/bge-large-en")
model.to(device)
# Tokenize sentences

@app.route("/")
def health():
    return "hello", 200


class EncodeRequest(BaseModel):
    input: List[str]

@app.post("/encode")
async def encode(encodingRequest: EncodeRequest):
    encoded_inputs = tokenizer.batch_encode_plus(
        encodingRequest.input,
        padding=True,
        truncation=True,
        max_length=512,
        add_special_tokens=True,
        return_tensors="pt",
    ).to(device)
        # for s2p(short query to long passage) retrieval task, add an instruction to query (not add instruction for passages)
        # encoded_input = tokenizer([instruction + q for q in queries], padding=True, truncation=True, return_tensors='pt')

        # Compute token embeddings
    with torch.no_grad():
        model_output = model(**encoded_inputs)
        # Perform pooling. In this case, cls pooling.
        sentence_embeddings = model_output[0][:, 0]
    # normalize embeddings
    sentence_embeddings = torch.nn.functional.normalize(sentence_embeddings, p=2, dim=1)
    
    return JSONResponse(content={"embeddings": sentence_embeddings[0].tolist()})


if __name__ == "__main__":
    # Run the app on localhost (127.0.0.1) and port 5000
   uvicorn.run("embeddings:app", host="127.0.0.1", port=5000, reload=True)

