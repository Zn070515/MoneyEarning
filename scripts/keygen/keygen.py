#!/usr/bin/env python3
"""
MoneyEarning License Key Generator
Usage: python keygen.py --fingerprint <HASH> --tier <pro|std> [--expiry YYYY-MM-DD] [--features feat1,feat2]
"""

import argparse
import base64
import json
import sys
from datetime import date
from cryptography.hazmat.primitives import hashes, serialization
from cryptography.hazmat.primitives.asymmetric import padding
from cryptography.hazmat.backends import default_backend


def load_private_key(path="private_key.pem"):
    with open(path, "rb") as f:
        return serialization.load_pem_private_key(f.read(), password=None, backend=default_backend())


def generate_license(fingerprint, tier, expiry, features, private_key):
    payload = {
        "fingerprint": fingerprint,
        "tier": tier,
        "expiry": expiry,  # None = perpetual
        "issued_at": date.today().isoformat(),
        "features": features or [],
    }
    payload_json = json.dumps(payload, separators=(",", ":"))
    payload_b64 = base64.b64encode(payload_json.encode()).decode()

    signature = private_key.sign(
        payload_json.encode(),
        padding.PKCS1v15(),
        hashes.SHA256(),
    )
    sig_b64 = base64.b64encode(signature).decode()

    return f"{payload_b64}.{sig_b64}"


def main():
    parser = argparse.ArgumentParser(description="MoneyEarning License Key Generator")
    parser.add_argument("--fingerprint", required=True, help="Machine fingerprint hash (SHA-256)")
    parser.add_argument("--tier", required=True, choices=["pro", "std"], help="License tier")
    parser.add_argument("--expiry", default=None, help="Expiry date YYYY-MM-DD (omit for perpetual)")
    parser.add_argument("--features", default=None, help="Comma-separated feature flags")
    parser.add_argument("--key-file", default=None, help="Path to private key PEM file")

    args = parser.parse_args()

    key_path = args.key_file or "private_key.pem"
    try:
        private_key = load_private_key(key_path)
    except FileNotFoundError:
        print(f"Error: Private key not found at '{key_path}'", file=sys.stderr)
        print("Generate keys first: openssl genpkey -algorithm RSA -out private_key.pem -pkeyopt rsa_keygen_bits:4096", file=sys.stderr)
        sys.exit(1)

    features = [f.strip() for f in args.features.split(",")] if args.features else []

    if args.expiry:
        try:
            date.fromisoformat(args.expiry)
        except ValueError:
            print("Error: Invalid expiry date format. Use YYYY-MM-DD.", file=sys.stderr)
            sys.exit(1)

    license_key = generate_license(
        args.fingerprint,
        args.tier,
        args.expiry,
        features,
        private_key,
    )

    print(license_key)
    print(f"\nTier: {args.tier}")
    print(f"Fingerprint: {args.fingerprint}")
    print(f"Expiry: {args.expiry or 'Perpetual'}")
    print(f"Features: {features or 'Default'}")


if __name__ == "__main__":
    main()
