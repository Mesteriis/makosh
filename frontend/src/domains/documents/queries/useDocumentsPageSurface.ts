import { computed } from 'vue'
import { useDocumentsStore } from '../stores/documents'
import {
  useDocumentProcessingJobsQuery,
  useRetryDocumentProcessingJobMutation,
} from './useDocumentsQuery'
import type { DocumentProcessingJob, DocDisplayItem } from '../types/documents'

export function useDocumentsPageSurface() {
  const store = useDocumentsStore()
  const retryDocumentProcessing = useRetryDocumentProcessingJobMutation()
  const jobsQuery = useDocumentProcessingJobsQuery(50)

  const documentProcessingJobs = computed(() => jobsQuery.data.value?.items ?? [])
  const documents = computed<DocDisplayItem[]>(() =>
    documentProcessingJobs.value.map((job) => ({
      name: `${job.document_id} (${job.step})`,
      source: 'Макошь',
      project: job.status,
      type: job.step,
      date: job.queued_at,
      size: job.last_error_summary || 'No errors',
      icon: 'tabler:file-text',
      tone: job.status === 'succeeded' ? 'green' : job.status === 'failed' ? 'red' : 'amber',
    }))
  )

  async function handleRetry(job: DocumentProcessingJob) {
    if (store.retryingJobId === job.job_id) return
    store.setRetryingJobId(job.job_id)
    store.setDocumentsError('')
    try {
      await retryDocumentProcessing.mutateAsync({
        jobId: job.job_id,
        request: {
          command_id: `document-processing-retry-${Date.now()}-${job.job_id}`,
        },
      })
    } catch (error) {
      store.setDocumentsError(error instanceof Error ? error.message : 'Retry failed')
    }
    await jobsQuery.refetch()
    store.setRetryingJobId(null)
  }

  return {
    documentProcessingJobs,
    documents,
    handleRetry,
    isLoading: jobsQuery.isLoading,
    refetchJobs: jobsQuery.refetch,
    store,
  }
}
