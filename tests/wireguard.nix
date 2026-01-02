{ pkgs, ... }:

let
  # Use pre-built krait-agent binary from local build
  # This assumes `just release-agent` was run before the integration test
  #
  # TODO(aksiksi): Define this as a native Nix derivation instead. The tricky
  # part is working around the fact that Aya build.rs scripts do manual rebuilds
  # via rustup, which breaks with Nix.
  kraitAgent = pkgs.stdenv.mkDerivation {
    name = "krait-agent-prebuilt";
    src = ../target/x86_64-unknown-linux-musl/release;

    installPhase = ''
      mkdir -p $out/bin
      cp krait $out/bin/
      chmod +x $out/bin/krait
    '';

    # Make sure the binary exists after installation
    doInstallCheck = true;
    installCheckPhase = ''
      test -f $out/bin/krait
    '';
  };

  # Krait agent systemd service definition
  kraitAgentService = {
    description = "Krait eBPF Agent";
    after = [ "network.target" "wireguard-wg0.service" ];
    wants = [ "wireguard-wg0.service" ];
    wantedBy = [ "multi-user.target" ];
    path = [ pkgs.iproute2 ];

    serviceConfig = {
      Type = "simple";
      ExecStart = "${kraitAgent}/bin/krait --iface wg0";
      Restart = "on-failure";
      RestartSec = "5s";

      # Security settings
      NoNewPrivileges = true;
      PrivateTmp = true;
      ProtectSystem = "strict";
      ProtectHome = true;

      # eBPF requires some capabilities
      CapabilityBoundingSet = [ "CAP_SYS_ADMIN" "CAP_NET_ADMIN" "CAP_SYS_RESOURCE" ];
      AmbientCapabilities = [ "CAP_SYS_ADMIN" "CAP_NET_ADMIN" "CAP_SYS_RESOURCE" ];
    };

    environment = {
      RUST_LOG = "info";
    };
  };
in

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
      
      # Krait agent systemd service
      systemd.services.krait-agent = kraitAgentService;
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
      
      # Krait agent systemd service
      systemd.services.krait-agent = kraitAgentService;
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

    with subtest("Wait for Krait agent to start"):
        server.wait_for_unit("krait-agent.service")
        client.wait_for_unit("krait-agent.service")
        # Verify agents are running
        server.succeed("systemctl is-active krait-agent.service")
        client.succeed("systemctl is-active krait-agent.service")

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
