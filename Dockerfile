FROM python:3.9-slim as build
WORKDIR /dist
ARG TARGETPLATFORM
# Deps
COPY ./scripts/requirements.txt .
RUN pip3 install -r requirements.txt
# Dist + script
COPY ./target/dist/ .
COPY ./scripts/target.py .
RUN python3 ./target.py ${TARGETPLATFORM}
RUN chmod +x ./a-train

FROM --platform=${TARGETPLATFORM} scratch as runtime
COPY --from=build /dist/a-train /
VOLUME /data
WORKDIR /data
ENTRYPOINT [ "/a-train" ]