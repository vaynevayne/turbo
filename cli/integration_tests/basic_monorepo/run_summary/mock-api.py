import http.server
import socketserver
import json
import os
import threading
import argparse

POST_COUNTER = 0
PATCH_COUNTER = 0
POST_LOCK = threading.Lock()
PATCH_LOCK = threading.Lock()

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
        with POST_LOCK:
            filename = f"post-{POST_COUNTER}.json"
            POST_COUNTER += 1
        self._record_request(filename)
        self.send_response(200)
        self.send_header('Content-type', 'application/json')
        self.end_headers()

        if self.path != "/api/v0/spaces/front/runs":
            return

        # Return some json from /runs endpoint
        json_response = json.dumps({"id": RUN_ID})
        self.wfile.write(json_response.encode())

    def do_PATCH(self):
        global PATCH_COUNTER
        with PATCH_LOCK:
            filename = f"patch-{PATCH_COUNTER}.json"
            PATCH_COUNTER += 1

        self._record_request(filename)
        self.send_response(200)
        self.send_header('Content-type', 'application/json')
        self.end_headers()


    def _record_request(self, filename):
        content_length = int(self.headers['Content-Length'])
        body = self.rfile.read(content_length)
        url = self.path
        request_dict = json.loads(body.decode())
        response_dict = {
            'requestUrl': url,
            'requestBody': request_dict
        }

        print("writing to: " + str(filename))
        with open(filename, "w") as f:
            json.dump(response_dict, f)

    # Supress all logs
    def log_message(self, format, *args):
        pass


with socketserver.ThreadingTCPServer(("", args.port), RequestHandler) as httpd:
    httpd.serve_forever()
