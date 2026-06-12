FROM nginx:alpine
COPY index.html login.html style.css app.js guestkit-auth.js guestkit-console.js guestkit-ai.js guestkit-features.js ironwolf-tokens.css ironwolf-shell.css ironwolf-components.css ironwolf-overrides.css /usr/share/nginx/html/
COPY fonts/ /usr/share/nginx/html/fonts/
COPY img/ /usr/share/nginx/html/img/
COPY nginx.conf /etc/nginx/conf.d/default.conf
EXPOSE 80
