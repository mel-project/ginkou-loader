{ stdenv, system }
stdenv.mkDerivation {
  name = "ginkou";
  inherit system;

  buildInputs = [ melwalletd ginkou ginkou-loader ];
  buildCommand = ''
    # Copy in melwalletd
    cp ${melwalletd}/bin/melwalletd .

    # Copy in js dependencies
    #cp ${ginkou}/result/lib/node_modules .

    # Copy in built js
    mkdir public
    mkdir public/build
    cp ${ginkou}/ ./public/build

    # Copy in the runtime loader
    cp ${ginkou-loader}/bin/ .

    # Create a run script
    echo ginkou-loader --html_path ./ginkou --melwalletd_path ./melwalletd > run.sh
  '';
}
