# render tailwindcss
tailwind:
	cd frontend && ./tailwindcss -i styles.css -o ../backend/static/styles.css

# serve the server
serve: tailwind
	cd backend && cargo build --bin ribbit && sudo systemctl start docker && sudo docker compose up
