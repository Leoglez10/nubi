cask "nubi" do
  version "0.1.3"
  sha256 "146613bfb825d49d758efd719c1f6af348c9a637c8b3a819a5665eb8306f28d7"

  url "https://github.com/Leoglez10/nubi/releases/download/v#{version}/nubi-macos.dmg"
  name "Nubi"
  desc "Desktop app to learn AWS against a local emulator"
  homepage "https://leoglez10.github.io/nubi/"

  depends_on :macos

  app "Nubi.app"

  # ponytail: the build is unsigned, so drop the quarantine flag instead of making users click
  # "Open Anyway"; remove once releases are signed and notarized.
  postflight_steps do
    run "/usr/bin/xattr",
        args:           ["-dr", "com.apple.quarantine", "{{appdir}}/Nubi.app"],
        writable_paths: ["Nubi.app"],
        writable_base:  :appdir
  end
end
