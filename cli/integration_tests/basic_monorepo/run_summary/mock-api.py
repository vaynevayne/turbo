import http.server
import socketserver
import json
import os
import argparse

POST_COUNTER = 0
PATCH_COUNTER = 0

RUN_ID = "1234"

parser = argparse.ArgumentParser()
parser.add_argument("--port", type=int, default=8000)
args = parser.parse_args()

# Write the process ID to a file
with open("server.pid", "w") as f:
    f.write(str(os.getpid()) + "\n")

class RequestHandler(http.server.SimpleHTTPRequestHandler):
    def do_POST(self):
        global POST_COUNTER
        filename = f"post-{POST_COUNTER}.json"
        POST_COUNTER += 1
        if self.path != "/api/v0/spaces/front/runs":
            response = None
        else:
            response = {"id": RUN_ID}

        self._record_request(filename, response)
        self.send_response(200)
        self.send_header('Content-type', 'application/json')
        self.end_headers()

        # Return some json from /runs endpoint
        if response != None:
          self.wfile.write(json.dumps(response).encode())

    def do_PATCH(self):
        global PATCH_COUNTER
        filename = f"patch-{PATCH_COUNTER}.json"
        self._record_request(filename)
        self.send_response(200)
        self.send_header('Content-type', 'application/json')
        self.end_headers()

    def do_PUT(self):
      pass

    def do_GET(self):
      pass


    def _record_request(self, filename, response = None):
        content_length = int(self.headers['Content-Length'])
        body = self.rfile.read(content_length)
        url = self.path
        request_dict = json.loads(body.decode())

        # TODO: record request headers here too?
        record_dict = {
            'requestUrl': url,
            'requestBody': request_dict
        }

        if response != None:
          record_dict['response'] = response

        print("writing to: " + str(filename))
        with open(filename, "w") as f:
            json.dump(record_dict, f)

with socketserver.TCPServer(("", args.port), RequestHandler) as httpd:
    httpd.serve_forever()
