
curl -X PUT http://localhost:9200/_template/oura.sink.v0 \
   -H 'Content-Type: application/json' \
   -d @index-template.json
