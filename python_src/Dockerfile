FROM python:3.8-slim-buster

WORKDIR /app

COPY . .

RUN pip install --upgrade pip

RUN pip install -r requirements.txt

EXPOSE 5000

CMD ["uvicorn", "embeddings:app", "--host", "0.0.0.0", "--port", "5000"]
