from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from urllib.parse import parse_qs, urlparse
import threading
import time


class KeyValueStore:
    def __init__(self):
        self.data = {}
        self.lock = threading.Lock()

    def put(self, key, value):
        if not isinstance(value, (int, float, str)):
            raise ValueError(
                "Invalid value type. Allowed types are int, float, or str."
            )
        with self.lock:
            self.data[key] = value

    def get(self, key):
        with self.lock:
            return self.data.get(key, None)

    def print(self):
        for key, value in self.data.items():
            print(f"Key: {key}, Value: {value}")


store: KeyValueStore = KeyValueStore()


class RequestHandler(BaseHTTPRequestHandler):
    def do_GET(self):
        start_time = time.time()
        get_path: str = "/get?key="
        if self.path.startswith(get_path):
            parsed_url = urlparse(self.path)
            query_params = parse_qs(parsed_url.query)
            key = query_params.get("key", [None])[0]

            self.send_response(200)
            self.send_header("Content-type", "text/plain")
            self.end_headers()

            value = store.get(key)

            self.wfile.write(str(value).encode() if value else b"Key not found")
            end_time = time.time()
            elapsed_time = end_time - start_time
            print(f"Elapsed time GET: {elapsed_time * 1000000}µs")
        else:
            self.send_response(404)
            self.send_header("Content-type", "text/plain")
            self.end_headers()
            self.wfile.write(b"Page not found")

    def do_PUT(self):
        start_time = time.time()
        put_path: str = "/put?key="
        if self.path.startswith(put_path):
            parsed_url = urlparse(self.path)
            query_params = parse_qs(parsed_url.query)
            key = query_params.get("key", [None])[0]
            value = query_params.get("value", [None])[0]

            store.put(key, value)

            self.send_response(200)
            self.send_header("Content-type", "text/plain")
            self.end_headers()

            self.wfile.write(b"Key-value pair added successfully")
            end_time = time.time()
            elapsed_time = end_time - start_time
            print(f"Elapsed time PUT: {elapsed_time * 1000000}µs")


def main(server_class=ThreadingHTTPServer, handler_class=RequestHandler, port=8080):
    start_time = time.time()
    server_address = ("", port)
    httpd = server_class(server_address, handler_class)
    print(f"Starting server on port {port}")
    end_time = time.time()
    elapsed_time = end_time - start_time
    print(f"Elapsed time starting server: {elapsed_time * 1000000}µs")
    httpd.serve_forever()


if __name__ == "__main__":
    main()