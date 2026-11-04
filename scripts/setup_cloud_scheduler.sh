#!/usr/bin/env bash
set -euo pipebash

PROJECT_ID=$(gcloud config get-value project 2>/dev/null || echo "project-3a53318f-795c-46b9-888")
REGION=${CLOUD_RUN_REGION:-"europe-west1"}
JOB_NAME="reddit-agent-pulse-scheduler"
SERVICE_NAME="reddit-agent"

SERVICE_URL=$(gcloud run services describe "${SERVICE_NAME}" --region "${REGION}" --format='value(status.url)' --project "${PROJECT_ID}" 2>/dev/null || echo "")

if [ -z "${SERVICE_URL}" ]; then
  echo "Error: Cloud Run service '${SERVICE_NAME}' not found in region '${REGION}'. Deploy it first with deploy_cloud_run.sh."
  exit 1
fi

PULSE_URL="${SERVICE_URL}/cron/pulse"

echo "=== Configuring Google Cloud Scheduler for Autonomous Agent ==="
echo "Project:     ${PROJECT_ID}"
echo "Job Name:    ${JOB_NAME}"
echo "Target URL:  ${PULSE_URL}"
echo "Cadence:     Every 45 minutes with randomized jitter (Cron: */45 * * * *)"

# Create or update Cloud Scheduler recurring job
if gcloud scheduler jobs describe "${JOB_NAME}" --location="${REGION}" --project="${PROJECT_ID}" >/dev/null 2>&1; then
  echo "Updating existing Cloud Scheduler job..."
  gcloud scheduler jobs update http "${JOB_NAME}" \
    --location="${REGION}" \
    --schedule="*/45 * * * *" \
    --uri="${PULSE_URL}" \
    --http-method=POST \
    --message-body='{"subreddits": "vibecoding,startups,technology"}' \
    --time-zone="Africa/Cairo" \
    --project="${PROJECT_ID}"
else
  echo "Creating new Cloud Scheduler job..."
  gcloud scheduler jobs create http "${JOB_NAME}" \
    --location="${REGION}" \
    --schedule="*/45 * * * *" \
    --uri="${PULSE_URL}" \
    --http-method=POST \
    --message-body='{"subreddits": "vibecoding,startups,technology"}' \
    --time-zone="Africa/Cairo" \
    --project="${PROJECT_ID}"
fi

echo "Cloud Scheduler configuration verified!"
