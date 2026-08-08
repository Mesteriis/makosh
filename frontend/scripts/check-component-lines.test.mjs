import { describe, expect, it } from 'vitest';
import {
	checkedExtensions,
	classifyLineCount,
	isLineCountCheckedSourceFile,
	isProductionSourceFile
} from './check-component-lines.mjs';

describe('check-component-lines policy', () => {
	it('checks production Vue and TypeScript source files', () => {
		expect(checkedExtensions.has('.vue')).toBe(true);
		expect(checkedExtensions.has('.ts')).toBe(true);
		expect(checkedExtensions.has('.tsx')).toBe(true);
		expect(isProductionSourceFile('src/domains/communications/components/ComposeDrawer.vue')).toBe(true);
		expect(isProductionSourceFile('src/domains/communications/queries/realtimeMailPatches.ts')).toBe(true);
	});

	it('excludes test files from the production source gate', () => {
		expect(isProductionSourceFile('src/platform/bootstrap/realtime.test.ts')).toBe(false);
		expect(isProductionSourceFile('src/domains/foo/__tests__/foo.ts')).toBe(false);
	});

	it('excludes generated and test files from the line-count architecture gate', () => {
		expect(isLineCountCheckedSourceFile('src/platform/bootstrap/realtime.test.ts')).toBe(false);
		expect(isLineCountCheckedSourceFile('src/domains/foo/__tests__/foo.ts')).toBe(false);
		expect(isLineCountCheckedSourceFile('src/domains/foo/__tests__/foo.vue')).toBe(false);
		expect(isLineCountCheckedSourceFile('src/gen/makosh/signal_hub/v1/signal_hub_pb.ts')).toBe(false);
	});

	it('treats 700 lines as a failure and 1000 lines as critical', () => {
		expect(classifyLineCount(499)).toEqual({ warning: false, failure: false, critical: false });
		expect(classifyLineCount(500)).toEqual({ warning: true, failure: false, critical: false });
		expect(classifyLineCount(700)).toEqual({ warning: true, failure: true, critical: false });
		expect(classifyLineCount(1000)).toEqual({ warning: true, failure: true, critical: true });
	});
});
