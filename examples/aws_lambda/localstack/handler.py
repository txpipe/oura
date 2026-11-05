def handler(event, context):
    # Oura invokes the function once per chain event; the payload is the JSON record.
    print("oura event:", event)
    return {"statusCode": 200}
