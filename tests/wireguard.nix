{ pkgs, ... }:

{
  name = "wireguard-connectivity-test";

  nodes = {
    # --- Server VM ---
    server = { pkgs, ... }: {
      networking.firewall.allowedUDPPorts = [ 51820 ];
      networking.firewall.allowedTCPPorts = [ 5201 ]; # iperf3 port
      networking.wireguard.interfaces.wg0 = {
        ips = [ "10.10.0.1/24" ];
        listenPort = 51820;
        # For testing, we hardcode keys. In production, use privateKeyFile.
        privateKey = builtins.readFile ./server.priv; 
        peers = [{
          publicKey = builtins.readFile ./client.pub;
          allowedIPs = [ "10.10.0.2/32" ];
        }];
      };
      environment.systemPackages = with pkgs; [
        iperf3
        tcpdump
        iproute2
      ];
    };

    # --- Client VM ---
    client = { pkgs, ... }: {
      networking.wireguard.interfaces.wg0 = {
        ips = [ "10.10.0.2/24" ];
        privateKey = builtins.readFile ./client.priv; 
        peers = [{
          publicKey = builtins.readFile ./server.pub;
          # In NixOS tests, nodes can resolve each other by their hostname.
          endpoint = "server:51820"; 
          allowedIPs = [ "10.10.0.0/24" ];
          persistentKeepalive = 25;
        }];
      };
      environment.systemPackages = with pkgs; [
        iperf3
        tcpdump
        iproute2
      ];
    };
  };

  # The testScript is written in Python
  testScript = ''
    start_all()

    with subtest("Wait for Wireguard services to start"):
        server.wait_for_unit("wireguard-wg0.service")
        client.wait_for_unit("wireguard-wg0.service")
        
        # Give additional time for interfaces to come up
        server.sleep(2)
        client.sleep(2)

    with subtest("Verify interface configuration"):
        # Verify interfaces have correct IP addresses
        server.succeed("ip addr show wg0 | grep '10.10.0.1/24'")
        client.succeed("ip addr show wg0 | grep '10.10.0.2/24'")
        
        # Show interface status for debugging
        print("Server wg0:", server.succeed("ip addr show wg0"))
        print("Client wg0:", client.succeed("ip addr show wg0"))

    with subtest("Establish Wireguard tunnel"):
        # Trigger handshake by having client send a packet first
        # This establishes the tunnel since client has the endpoint configured
        print("Initiating Wireguard handshake...")
        
        # Try ping until it succeeds to establish the tunnel
        client.wait_until_succeeds("ping -c 1 10.10.0.1", timeout=30)
        
        # Verify handshake occurred on both sides
        server.succeed("wg show | grep 'latest handshake'")
        client.succeed("wg show | grep 'latest handshake'")

    with subtest("Test tunnel connectivity"):
        # Test bidirectional connectivity
        client.succeed("ping -c 3 10.10.0.1")
        server.succeed("ping -c 3 10.10.0.2")

    with subtest("Test tunnel throughput"):
        server.succeed("iperf3 -s -D")
        server.sleep(1)
        client.succeed("iperf3 -c 10.10.0.1 -t 5")

    print("Wireguard tunnel test completed successfully!")
  '';
}
