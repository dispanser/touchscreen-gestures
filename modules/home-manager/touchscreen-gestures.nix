{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.programs.touchscreen-gestures;
  inherit (pkgs.stdenv.hostPlatform) system;

  tomlType = pkgs.formats.toml { };

  actionType = lib.types.submodule {
    options = {
      gesture = lib.mkOption {
        type = lib.types.listOf lib.types.str;
        description = "Gesture pattern sequence";
      };
      cmd = lib.mkOption {
        type = lib.types.nullOr lib.types.str;
        default = null;
        description = "Built-in command name (e.g., 'BothScreens')";
        apply = v: if v == null then null else v;
      };
      run = lib.mkOption {
        type = lib.types.nullOr lib.types.str;
        default = null;
        description = "Shell command to execute";
        apply = v: if v == null then null else v;
      };
    };
  };

  generatedConfig = tomlType.generate "touchscreen-gestures-config" {
    poll_interval_ms = cfg.pollIntervalMs;
    actions = map (action: {
      gesture = action.gesture;
    } // lib.optionalAttrs (action.cmd != null) { cmd = action.cmd; }
      // lib.optionalAttrs (action.run != null) { run = action.run; }) cfg.actions;
  };
in
{
  meta.maintainers = [];

  options.programs.touchscreen-gestures = {
    enable = lib.mkEnableOption "touchscreen-gestures daemon";

    package = lib.mkOption {
      type = lib.types.package;
      defaultText = lib.literalExpression "touchscreen-gestures package from flake";
      description = "touchscreen-gestures package to use";
    };

    pollIntervalMs = lib.mkOption {
      type = lib.types.int;
      default = 500;
      description = "Poll interval in milliseconds";
    };

    actions = lib.mkOption {
      type = lib.types.listOf actionType;
      default = [ ];
      description = "List of gesture actions";
    };

    configFile = lib.mkOption {
      type = lib.types.nullOr lib.types.path;
      default = null;
      description = "Path to an external TOML configuration file (if not using inline actions)";
    };

    # script = lib.mkOption {
    #   type = lib.types.lines;
    #   default = "";
    #   description = "Additional shell commands to run before starting touchscreen-gestures";
    # };
  };

  config = lib.mkIf cfg.enable {
    systemd.user.services.touchscreen-gestures = {
      Unit = {
        Description = "Touchscreen gesture recognition daemon";
      };
      Service = {
        Environment = ''RUST_LOG=debug'';
        ExecStart = "${lib.getExe cfg.package} --config ${
          if cfg.configFile != null then cfg.configFile else generatedConfig
        }";
        Restart = "on-failure";
        RestartSec = 5;
      };
      Install = {
        WantedBy = [ "graphical-session.target" ];
      };
    };
  };
}
