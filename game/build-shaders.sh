#!/usr/bin/env bash

RED='\033[0;31m'
GREEN='\033[0;32m'
LIGHT_GREEN='\033[1;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

SCRIPT_DIR=$(cd -P -- "$(dirname -- "$0")" && pwd -P)
cd "${SCRIPT_DIR}"

echo -e "   ${LIGHT_GREEN}Compiling GLSL shaders${NC}"
for f in $(
    find src -type f -name '*.vert'
    find src -type f -name '*.frag'
)
do
    if [[ ! ("${f}" =~ "^.*\\.spv$") ]]
    then
        echo "Compiling '${f}'"
        glslc -o "${f}.spv" "${f}" || (
            echo -e "${RED}error${NC}: failed to compile '${f}'"
            exit 1
        ) || exit 1
    fi
done