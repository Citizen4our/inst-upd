build:
    cross build --release
    
c_p:
    @echo "Copying binary file to the raspberry pi"
    robo flash --file target/aarch64-unknown-linux-gnu/release/inst-upd -r -f
bf:
    @echo "Compiling and flashing binary file to the raspberry pi"
    robo flash -r -f

ssh:
    @echo "SSH into the raspberry pi"
    ssh citizen4our@192.168.50.52