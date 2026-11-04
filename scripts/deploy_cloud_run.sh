#!/usr/bin/env bash
set -euo pipebash

PROJECT_ID=$(gcloud config get-value project 2>/dev/null || echo "project-3a53318f-795c-46b9-888")
REGION=${CLOUD_RUN_REGION:-"europe-west1"}
SERVICE_NAME="reddit-agent"
IMAGE_TAG="gcr.io/${PROJECT_ID}/${SERVICE_NAME}:latest"

echo "=== Deploying Autonomous Reddit Agent to Google Cloud Run ==="
echo "Project:  ${PROJECT_ID}"
echo "Region:   ${REGION}"
echo "Service:  ${SERVICE_NAME}"
echo "Image:    ${IMAGE_TAG}"

echo "Step 1: Building container with Google Cloud Build..."
gcloud builds submit --tag "${IMAGE_TAG}" --project "${PROJECT_ID}"

echo "Step 2: Deploying to Cloud Run..."
gcloud run deploy "${SERVICE_NAME}" \
  --image "${IMAGE_TAG}" \
  --region "${REGION}" \
  --platform managed \
  --allow-unauthenticated \
  --memory 512Mi \
  --cpu 1 \
  --concurrency 10 \
  --min-instances 0 \
  --max-instances 2 \
  --set-env-vars "RUST_LOG=info,EVAL_MODE=cloud" \
  --project "${PROJECT_ID}"

echo "Deployment complete! Service URL:"
gcloud run services describe "${SERVICE_NAME}" --region "${REGION}" --format='value(status.url)' --project "${PROJECT_ID}"
