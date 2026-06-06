FROM nginx:alpine
COPY index.html style.css app.js /usr/share/nginx/html/
COPY img/ /usr/share/nginx/html/img/
COPY nginx.conf /etc/nginx/conf.d/default.conf
EXPOSE 80
