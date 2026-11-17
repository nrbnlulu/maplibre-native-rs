#pragma once

#include <string>
#include <dirent.h>
#include <fstream>
#include <iterator>

namespace {
inline const std::string& androidTrustBundle() {
    static const std::string bundle = [] {
        std::string certificates;
        for (const char* path : {"/apex/com.android.conscrypt/cacerts", "/system/etc/security/cacerts"}) {
            DIR* directory = opendir(path);
            if (!directory) continue;
            while (auto* entry = readdir(directory)) {
                if (entry->d_name[0] == '.') continue;
                std::ifstream file(std::string(path) + "/" + entry->d_name);
                std::string pem((std::istreambuf_iterator<char>(file)), {});
                if (pem.find("-----BEGIN CERTIFICATE-----") != std::string::npos) {
                    certificates += pem;
                    certificates += '\n';
                }
            }
            closedir(directory);
        }
        return certificates;
    }();
    return bundle;
}
}
