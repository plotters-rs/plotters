import sys
import mimetypes
mimetypes.add_type(".js", "text/javascript")
PORT = 8000
if sys.version_info[0] < 3:
	import SimpleHTTPServer
	import SocketServer
	Handler = SimpleHTTPServer.SimpleHTTPRequestHandler
	Handler.extensions_map[".js"] = "text/javascript"
	httpd = SocketServer.TCPServer(("", PORT), Handler)
	httpd.serve_forever()
else:
	from http.server import SimpleHTTPRequestHandler, HTTPServer
	SimpleHTTPRequestHandler.extensions_map[".js"] = "text/javascript"
	httpd = HTTPServer(("", PORT),SimpleHTTPRequestHandler)
	httpd.serve_forever()
	
