./target/debug/client -m "fortune" -r 10 -d 1000 --wait &
./target/debug/client -m "fortune" -r 30 -d 300 --wait &

./target/debug/client -m "increment" -r 50 -d 100 --wait &
./target/debug/client -m "increment" -r 20 -d 500 --wait &
./target/debug/client -m "increment" -r 100 -d 50 --wait &

./target/debug/client -m "upload test" -r 10 -d 1000 --wait &
./target/debug/client -m "upload hello" -r 100 -d 100 --wait &
./target/debug/client -m "upload world" -r 100 -d 100 --wait &

./target/debug/client -m "download test" -r 10 -d 1000 --wait &
./target/debug/client -m "download hello" -r 100 -d 100 --wait &
./target/debug/client -m "download world" -r 100 -d 100 --wait &

./target/debug/client -m "compute 3000" -r 100 -d 100 --wait &
./target/debug/client -m "compute 30000" -r 10 -d 1000 --wait &

./target/debug/client -m "counter" -r 100 -d 100 --wait

