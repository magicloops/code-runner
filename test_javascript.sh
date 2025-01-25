#!/bin/bash

curl -X POST http://localhost:4000/run \
  -H 'Content-Type: application/json' \
  -d '{
        "image": "node:latest",
        "payload": {
          "language": "node",
          "files": [
            {
              "name": "script.js",
              "content": "console.log(\"Hello World from Node\");"
            }
          ]
        }
      }'
