import { DevelopmentOwnerDeviceProofV1 } from './developmentOwnerDeviceProof'
import {
	BrowserOwnerDeviceProofV1,
	type OwnerDeviceProofV1,
} from './ownerDeviceProof'

export function hasDevelopmentOwnerDeviceProofHostV1(): boolean {
	return import.meta.env.VITE_MAKOSH_DEV_OWNER_DEVICE_PROOF_HOST === '1'
}

export function createOwnerDeviceProofV1(): OwnerDeviceProofV1 {
	if (hasDevelopmentOwnerDeviceProofHostV1()) {
		return new DevelopmentOwnerDeviceProofV1()
	}
	return new BrowserOwnerDeviceProofV1()
}
