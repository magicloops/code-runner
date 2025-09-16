#!/bin/sh

if test $# -ne 1; then
    echo "Usage: $0 <runner-url>" 1>&2
    echo "" 1>&2
    echo "    $0 http://localhost:4000" 1>&2
    echo "    $0 https://code-runner-magicloops.fra0.kraft.host" 1>&2
    exit 1
fi

runner_url="$1"

curl -s -X POST "$runner_url"/run \
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
