#include <array>
#include <cstddef>
#include <cstdio>
#include <cstdlib>
#include <filesystem>
#include <fstream>
#include <iostream>
#include <linux/sysinfo.h>
#include <memory>
#include <string>
#include <sys/sysinfo.h>
#include <sys/utsname.h>

struct SystemInfo {
  std::string hostname;
  std::string distro;
  std::string init_system;
  std::string kernel_version;
  int native_packages;
  int flatpak_packages;
  unsigned long mem_total;
  unsigned long mem_free;
};

std::string exec(const char *cmd) {
  std::array<char, 128> buffer;
  std::string res;
  std::unique_ptr<FILE, decltype(&pclose)> pipe(popen(cmd, "r"), pclose);

  while (fgets(buffer.data(), buffer.size(), pipe.get()) != nullptr) {
    res += buffer.data();
  }

  return res;
}

std::string get_distro_name() {
  std::ifstream os_release("/etc/os-release");
  std::string line;

  while (std::getline(os_release, line)) {
    if (line.starts_with("PRETTY_NAME=")) {
      return line.substr(13, line.length() - 14);
    }
  }

  return "unknown";
}

std::string get_init_system() {
  if (std::filesystem::exists("/run/systemd/system")) {
    return "systemd";
  }

  if (std::filesystem::exists("/sbin/openrc")) {
    return "openrc";
  }

  if (std::filesystem::exists("/etc/runit/runsvdir/default")) {
    return "runit";
  }

  if (std::filesystem::exists("/etc/s6")) {
    return "s6";
  }

  if (std::filesystem::exists("/etc/init.d") &&
      std::filesystem::exists("/sbin/rc")) {
    return "sysvinit";
  }

  if (std::filesystem::exists("/etc/dinit")) {
    return "dinit";
  }

  if (std::filesystem::exists("/run/shepherd")) {
    return "shepherd";
  }

  return "other";
}

int count_packages() {
  // Debian/Ubuntu
  if (std::filesystem::exists("/usr/bin/dpkg")) {
    return std::stoi(exec("dpkg -l | grep '^ii' | wc -l"));
  }

  // Arch
  if (std::filesystem::exists("/usr/bin/pacman")) {
    return std::stoi(exec("pacman -Q | wc -l"));
  }

  // Fedora
  if (std::filesystem::exists("/usr/bin/rpm")) {
    return std::stoi(exec("rpm -qa | wc -l"));
  }

  // Void
  if (std::filesystem::exists("/usr/bin/xbps-query")) {
    return std::stoi(exec("xbps-query -l | wc -l"));
  }

  // Gentoo
  if (std::filesystem::exists("/usr/bin/qlist")) {
    return std::stoi(exec("qlist -I | wc -l"));
  }

  // Alpine
  if (std::filesystem::exists("/sbin/apk")) {
    return std::stoi(exec("apk info | wc -l"));
  }

  // NixOS
  if (std::filesystem::exists("/run/current-system/sw/bin/nix")) {
    return std::stoi(
        exec("nix-store -q --requisites /run/current-system/sw | wc -l"));
  }

  return 0;
}

SystemInfo collect_system_info() {
  SystemInfo info;
  struct utsname uts;
  uname(&uts);
  info.hostname = uts.nodename;
  info.distro = get_distro_name();
  info.init_system = get_init_system();
  info.kernel_version = uts.release;
  info.native_packages = count_packages();

  std::string flatpak_count = exec("flatpak list | wc -l");
  info.flatpak_packages = std::stoi(flatpak_count);

  struct sysinfo si;

  if (sysinfo(&si) == 0) {
    info.mem_total = si.totalram * si.mem_unit / 1024 / 1024;
    info.mem_free = si.freeram * si.mem_unit / 1024 / 1024;
  }

  return info;
}

int main() {
  auto info = collect_system_info();

  // We store logo lines in an array for easier access and efficiency
  std::array<std::string, 6> logo = {".------.", "|H.--. |", "| :/\\: |",
                                     "| (__) |", "| '--'H|", "`------'"};

  // Also for info
  std::array<std::string, 6> info_lines = {
      "\033[1;34mhostname: \033[0m" + info.hostname,
      "\033[1;34mdistro: \033[0m" + info.distro,
      "\033[1;34mkernel: \033[0m" + info.kernel_version,
      "\033[1;34minit: \033[0m" + info.init_system,
      "\033[1;34mpackages: \033[0m" + std::to_string(info.native_packages) +
          " native | " + std::to_string(info.flatpak_packages) + " flatpak",
      "\033[1;34mmemory: \033[0m" + std::to_string(info.mem_free) + "MB | " +
          std::to_string(info.mem_total) + "MB"};

  for (size_t i = 0; i < logo.size(); i++) {
    std::cout << "\033[1;35m" << logo[i] << "\033[0m";

    if (i < info_lines.size()) {
      std::cout << "    " << info_lines[i];
    }

    std::cout << "\n";
  }
}
