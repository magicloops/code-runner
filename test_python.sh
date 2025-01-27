#!/bin/bash 

curl -X POST http://localhost:4000/run \
  -H 'Content-Type: application/json' \
  -d '{
        "image": "python:latest",
        "payload": {
          "language": "python",
          "files": [
            {
              "name": "script.py",
              "content": "print(\"Hello World from Python\")"
            }
          ]
        }
      }'
