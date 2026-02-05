#!/usr/bin/env bash
# Chrysalis installer script for macOS and Linux

set -euo pipefail

# Repository information
REPO="ZenAI-International-Corp/chrysalis"
BINARY_NAME="chrysalis"

# Reset
Color_Off=''

# Regular Colors
Red=''
Green=''
Dim=''

# Bold
Bold_White=''
Bold_Green=''

if [[ -t 1 ]]; then
    # Reset
    Color_Off='\033[0m'

    # Regular Colors
    Red='\033[0;31m'
    Green='\033[0;32m'
    Dim='\033[0;2m'

    # Bold
    Bold_Green='\033[1;32m'
    Bold_White='\033[1m'
fi

error() {
    echo -e "${Red}error${Color_Off}:" "$@" >&2
    exit 1
}

info() {
    echo -e "${Dim}$@ ${Color_Off}"
}

info_bold() {
    echo -e "${Bold_White}$@ ${Color_Off}"
}

success() {
    echo -e "${Green}$@ ${Color_Off}"
}

# Detect platform
platform=$(uname -ms)

case $platform in
'Darwin x86_64')
    target=darwin-amd64
    ;;
'Darwin arm64')
    target=darwin-arm64
    ;;
'Linux aarch64' | 'Linux arm64')
    target=linux-arm64
    ;;
'Linux x86_64' | *)
    target=linux-amd64
    ;;
esac

if [[ $target = darwin-amd64 ]]; then
    # Is this process running in Rosetta?
    if [[ $(sysctl -n sysctl.proc_translated 2>/dev/null) = 1 ]]; then
        target=darwin-arm64
        info "Your shell is running in Rosetta 2. Downloading chrysalis for $target instead"
    fi
fi

# Get latest version or use provided version
if [[ $# -gt 0 ]]; then
    VERSION=$1
else
    info "Fetching latest version..."
    VERSION=$(curl -s "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/') ||
        error "Failed to fetch latest version"

    if [ -z "$VERSION" ]; then
        error "Could not determine latest version"
    fi
fi

# Set up install directory
install_env=CHRYSALIS_INSTALL
bin_env=\$$install_env/bin
install_dir=${!install_env:-$HOME/.chrysalis}
bin_dir=$install_dir/bin
exe=$bin_dir/$BINARY_NAME

if [[ ! -d $bin_dir ]]; then
    mkdir -p "$bin_dir" ||
        error "Failed to create install directory \"$bin_dir\""
fi

# Download
asset_name="${BINARY_NAME}-${target}"
download_url="https://github.com/$REPO/releases/download/$VERSION/${asset_name}.tar.gz"

info "Downloading Chrysalis $VERSION for $target..."

curl --fail --location --progress-bar --output "$exe.tar.gz" "$download_url" ||
    error "Failed to download chrysalis from \"$download_url\""

# Extract
tar -xzf "$exe.tar.gz" -C "$bin_dir" ||
    error "Failed to extract chrysalis"

# Move binary to correct location
mv "$bin_dir/$BINARY_NAME" "$exe" ||
    error "Failed to move extracted chrysalis to destination"

chmod +x "$exe" ||
    error "Failed to set permissions on chrysalis executable"

rm "$exe.tar.gz"

tildify() {
    if [[ $1 = $HOME/* ]]; then
        local replacement=\~/
        echo "${1/$HOME\//$replacement}"
    else
        echo "$1"
    fi
}

success "chrysalis was installed successfully to $Bold_Green$(tildify "$exe")"

if command -v chrysalis >/dev/null; then
    echo "Run 'chrysalis --help' to get started"
    exit
fi

refresh_command=''

tilde_bin_dir=$(tildify "$bin_dir")
quoted_install_dir=\"${install_dir//\"/\\\"}\"

if [[ $quoted_install_dir = \"$HOME/* ]]; then
    quoted_install_dir=${quoted_install_dir/$HOME\//\$HOME/}
fi

echo

case $(basename "$SHELL") in
fish)
    commands=(
        "set --export $install_env $quoted_install_dir"
        "set --export PATH $bin_env \$PATH"
    )

    fish_config=$HOME/.config/fish/config.fish
    tilde_fish_config=$(tildify "$fish_config")

    if [[ -w $fish_config ]]; then
        # Check if chrysalis is already configured
        if ! grep -q "# chrysalis" "$fish_config"; then
            {
                echo -e '\n# chrysalis'

                for command in "${commands[@]}"; do
                    echo "$command"
                done
            } >>"$fish_config"

            info "Added \"$tilde_bin_dir\" to \$PATH in \"$tilde_fish_config\""

            refresh_command="source $tilde_fish_config"
        else
            info "Chrysalis is already configured in \"$tilde_fish_config\""
            refresh_command="source $tilde_fish_config"
        fi
    else
        echo "Manually add the directory to $tilde_fish_config (or similar):"

        for command in "${commands[@]}"; do
            info_bold "  $command"
        done
    fi
    ;;
zsh)
    commands=(
        "export $install_env=$quoted_install_dir"
        "export PATH=\"$bin_env:\$PATH\""
    )

    zsh_config=$HOME/.zshrc
    tilde_zsh_config=$(tildify "$zsh_config")

    if [[ -w $zsh_config ]]; then
        # Check if chrysalis is already configured
        if ! grep -q "# chrysalis" "$zsh_config"; then
            {
                echo -e '\n# chrysalis'

                for command in "${commands[@]}"; do
                    echo "$command"
                done
            } >>"$zsh_config"

            info "Added \"$tilde_bin_dir\" to \$PATH in \"$tilde_zsh_config\""

            refresh_command="exec $SHELL"
        else
            info "Chrysalis is already configured in \"$tilde_zsh_config\""
            refresh_command="exec $SHELL"
        fi
    else
        echo "Manually add the directory to $tilde_zsh_config (or similar):"

        for command in "${commands[@]}"; do
            info_bold "  $command"
        done
    fi
    ;;
bash)
    commands=(
        "export $install_env=$quoted_install_dir"
        "export PATH=\"$bin_env:\$PATH\""
    )

    bash_configs=(
        "$HOME/.bash_profile"
        "$HOME/.bashrc"
    )

    if [[ ${XDG_CONFIG_HOME:-} ]]; then
        bash_configs+=(
            "$XDG_CONFIG_HOME/.bash_profile"
            "$XDG_CONFIG_HOME/.bashrc"
            "$XDG_CONFIG_HOME/bash_profile"
            "$XDG_CONFIG_HOME/bashrc"
        )
    fi

    set_manually=true
    for bash_config in "${bash_configs[@]}"; do
        tilde_bash_config=$(tildify "$bash_config")

        if [[ -w $bash_config ]]; then
            # Check if chrysalis is already configured
            if ! grep -q "# chrysalis" "$bash_config"; then
                {
                    echo -e '\n# chrysalis'

                    for command in "${commands[@]}"; do
                        echo "$command"
                    done
                } >>"$bash_config"

                info "Added \"$tilde_bin_dir\" to \$PATH in \"$tilde_bash_config\""

                refresh_command="source $bash_config"
                set_manually=false
                break
            else
                info "Chrysalis is already configured in \"$tilde_bash_config\""
                refresh_command="source $bash_config"
                set_manually=false
                break
            fi
        fi
    done

    if [[ $set_manually = true ]]; then
        echo "Manually add the directory to $tilde_bash_config (or similar):"

        for command in "${commands[@]}"; do
            info_bold "  $command"
        done
    fi
    ;;
*)
    echo 'Manually add the directory to ~/.bashrc (or similar):'
    info_bold "  export $install_env=$quoted_install_dir"
    info_bold "  export PATH=\"$bin_env:\$PATH\""
    ;;
esac

echo
info "To get started, run:"
echo

if [[ $refresh_command ]]; then
    info_bold "  $refresh_command"
fi

info_bold "  chrysalis --help"
