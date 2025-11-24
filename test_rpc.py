import socket
import json
import sys

PORT = 4000
MAGNET = "magnet:?xt=urn:btih:dd8255ecdc7ca55fb0bbf81323d87062db1f6d1c&dn=Big+Buck+Bunny&tr=udp%3A%2F%2Fexplodie.org%3A6969&tr=udp%3A%2F%2Ftracker.coppersurfer.tk%3A6969&tr=udp%3A%2F%2Ftracker.empire-js.us%3A1337&tr=udp%3A%2F%2Ftracker.leechers-paradise.org%3A6969&tr=udp%3A%2F%2Ftracker.opentrackr.org%3A1337&tr=wss%3A%2F%2Ftracker.btorrent.xyz&tr=wss%3A%2F%2Ftracker.fastcast.nz&tr=wss%3A%2F%2Ftracker.openwebtorrent.com&ws=https%3A%2F%2Fwebtorrent.io%2Ftorrents%2F&xs=https%3A%2F%2Fwebtorrent.io%2Ftorrents%2Fbig-buck-bunny.torrent"

def send_rpc(method, params=None):
    req = {
        "jsonrpc": "2.0",
        "id": 1,
        "method": method,
    }
    if params:
        req["params"] = params
    
    try:
        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
            s.connect(('127.0.0.1', PORT))
            s.sendall(json.dumps(req).encode('utf-8'))
            
            data = s.recv(4096)
            
            print(f"Response for {method}: {data.decode('utf-8')}")
            return json.loads(data.decode('utf-8'))
    except (socket.error, OSError) as e:
        print(f"Network Error sending {json.dumps(req)}: {e}")
        return None
    except Exception as e:
        print(f"Unexpected Error: {e}")
        return None

if __name__ == "__main__":
    print("Adding Torrent...")
    send_rpc("AddTorrent", {"magnet": MAGNET})

    print("\nListing Torrents...")
    res = send_rpc("ListTorrents")
    print(json.dumps(res, indent=2))
