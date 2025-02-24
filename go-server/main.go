package main

import (
	"context"
	"crypto/tls"
	"crypto/x509"
	"encoding/pem"
	"fmt"
	"github.com/sethvargo/go-envconfig"
	"log"
	"net/http"
	"os"
)

type ServerConfig struct {
	Message     string `env:"SERVER_MESSAGE"`
	Mode        string `env:"SERVER_MODE"`
	Port        string `env:"SERVER_PORT"`
	ServerCert  string `env:"TLS_SERVER_CERT"`
	ServerKey   string `env:"TLS_SERVER_KEY"`
	ClientRoots string `env:"TLS_CLIENT_ROOTS"`
}

func makeTlsConfig(serverConfig *ServerConfig) *tls.Config {
	if serverConfig.Mode == "HTTP" {
		return nil
	}

	cfg := &tls.Config{
		MinVersion:       tls.VersionTLS12,
		CurvePreferences: []tls.CurveID{tls.CurveP521, tls.CurveP384, tls.CurveP256},
		CipherSuites: []uint16{
			tls.TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384,
			tls.TLS_ECDHE_RSA_WITH_AES_256_CBC_SHA,
			tls.TLS_RSA_WITH_AES_256_GCM_SHA384,
			tls.TLS_RSA_WITH_AES_256_CBC_SHA,
			tls.TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256,
			tls.TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256,
		},
		NextProtos: []string{"h2", "http/1.1", "http/1.0"},
	}

	if serverConfig.ClientRoots != "" {
		cfg.ClientAuth = tls.RequireAndVerifyClientCert
		cfg.ClientCAs = loadClientCerts(serverConfig.ClientRoots)
	} else {
		cfg.ClientAuth = tls.NoClientCert
	}

	return cfg
}

func loadClientCerts(path string) *x509.CertPool {
	certPool := x509.NewCertPool()

	rawPem, err := os.ReadFile(path)
	if err != nil {
		log.Fatalf("Error reading client allowed roots (%s): %v\n", path, err)
	}

	var added uint = 0
	for {
		var certDERBlock *pem.Block
		certDERBlock, rawPem = pem.Decode(rawPem)
		if certDERBlock == nil {
			break
		}
		cert, err := x509.ParseCertificate(certDERBlock.Bytes)
		if err != nil {
			log.Fatalf("Error parsing X509 certificate (%s): %v\n", path, err)
		}

		certPool.AddCert(cert)
		added += 1
	}

	log.Printf("Loaded client cert pool (%s): %v certificates\n", path, added)

	return certPool
}

func main() {
	ctx := context.Background()

	var c ServerConfig
	if err := envconfig.Process(ctx, &c); err != nil {
		log.Fatalf("Failed to read configuration: %v\n", err)
	}

	if c.Mode == "" {
		c.Mode = "HTTP"
	} else if c.Mode != "HTTP" && c.Mode != "HTTPS" {
		log.Fatalf("Invalid mode: %s, expected either HTTP or HTTPS\n", c.Mode)
	}

	if c.Message == "" {
		c.Message = "Hello from remote!"
	}

	if c.Port == "" {
		if c.Mode == "HTTP" {
			c.Port = "80"
		} else {
			c.Port = "443"
		}
	}

	log.Printf("Resolved server configuration: %+v\n", c)

	mux := http.NewServeMux()
	mux.HandleFunc("/", func(w http.ResponseWriter, req *http.Request) {
		log.Printf("Got a %s %s request from %s\n", req.Proto, req.Method, req.RemoteAddr)

		if c.Mode == "HTTPS" {
			w.Header().Add("Strict-Transport-Security", "max-age=63072000; includeSubDomains")
		}

		w.Header().Add("Content-Type", "text/plain")

		_, _ = w.Write([]byte(c.Message))
	})

	srv := &http.Server{
		Addr:      fmt.Sprintf(":%s", c.Port),
		Handler:   mux,
		TLSConfig: makeTlsConfig(&c),
	}

	var err error
	if c.Mode == "HTTP" {
		err = srv.ListenAndServe()
	} else {
		err = srv.ListenAndServeTLS(c.ServerCert, c.ServerKey)
	}

	if err != nil {
		log.Fatalf("Server failed: %v\n", err)
	}
}
