#!/bin/bash

# Get the JSON file path from the first argument
JSON_FILE=$1

# Read the JSON data from the file
JSON_PROOF=$(cat "$JSON_FILE")

curl -X POST \
  'http://127.0.0.1:45000' \
  --header 'Accept: */*' \
  --header 'Content-Type: application/json' \
  --data-raw "{
    \"jsonrpc\": \"2.0\",
    \"method\": \"twarb_sendProof\",
    \"params\": [
      {
        \"type\": \"SP1Proof\",
        \"identifier\": \"identifier1\",
        \"proof\": $JSON_PROOF
      }
    ],
    \"id\": 1
  }"