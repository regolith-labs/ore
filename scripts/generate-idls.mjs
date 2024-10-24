#!/usr/bin/env zx
import 'zx/globals';
import { generateIdl } from '@metaplex-foundation/shank-js';
import { getCargo } from './utils.mjs';

const binaryInstallDir = path.join(__dirname, '..', '.cargo');

const interfaceDir = path.join(__dirname, '..', 'api');
const cargo = getCargo('api');

await generateIdl({
  generator: 'shank',
  programName: cargo.package.name.replace(/-/g, '_'),
  programId: cargo.package.metadata.solana['program-id'],
  idlDir: interfaceDir,
  idlName: 'idl',
  programDir: interfaceDir,
  binaryInstallDir,
});
