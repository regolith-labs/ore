import 'zx/globals';
import { parse as parseToml } from '@iarna/toml';

process.env.FORCE_COLOR = 3;
process.env.CARGO_TERM_COLOR = 'always';

export const workingDirectory = (await $`pwd`.quiet()).toString().trim();

export function getAllProgramIdls() {
  return getAllProgramFolders().map((folder) =>
    path.join(workingDirectory, folder, 'idl.json')
  );
}

export function getExternalProgramOutputDir() {
  const config = getCargoMetadata()?.solana?.['external-programs-output'];
  return path.join(workingDirectory, config ?? 'target/deploy');
}

export function getExternalProgramAddresses() {
  const addresses = getProgramFolders().flatMap(
    (folder) => getCargoMetadata(folder)?.solana?.['program-dependencies'] ?? []
  );
  return Array.from(new Set(addresses));
}

export function getExternalAccountAddresses() {
  const addresses = getProgramFolders().flatMap(
    (folder) => getCargoMetadata(folder)?.solana?.['account-dependencies'] ?? []
  );
  return Array.from(new Set(addresses));
}

let didWarnAboutMissingPrograms = false;
export function getProgramFolders() {
  let programs;

  if (process.env.PROGRAMS) {
    try {
      programs = JSON.parse(process.env.PROGRAMS);
    } catch (error) {
      programs = process.env.PROGRAMS.split(/\s+/);
    }
  } else {
    programs = getAllProgramFolders();
  }

  const filteredPrograms = programs.filter((program) =>
    fs.existsSync(path.join(workingDirectory, program))
  );

  if (
    filteredPrograms.length !== programs.length &&
    !didWarnAboutMissingPrograms
  ) {
    didWarnAboutMissingPrograms = true;
    programs
      .filter((program) => !filteredPrograms.includes(program))
      .forEach((program) => {
        echo(chalk.yellow(`Program not found: ${workingDirectory}/${program}`));
      });
  }

  return filteredPrograms;
}

export function getAllProgramFolders() {
  return getCargo().workspace.members.filter((member) =>
    (getCargo(member).lib?.['crate-type'] ?? []).includes('cdylib')
  );
}

export function getCargo(folder) {
  return parseToml(
    fs.readFileSync(
      path.join(workingDirectory, folder ? folder : '.', 'Cargo.toml'),
      'utf8'
    )
  );
}

export function getCargoMetadata(folder) {
  const cargo = getCargo(folder);
  return folder ? cargo?.package?.metadata : cargo?.workspace?.metadata;
}

export function getSolanaVersion() {
  return getCargoMetadata()?.cli?.solana;
}

export function getToolchain(operation) {
  return getCargoMetadata()?.toolchains?.[operation];
}

export function getToolchainArgument(operation) {
  const channel = getToolchain(operation);
  return channel ? `+${channel}` : '';
}

export function cliArguments() {
  return process.argv.slice(3);
}

export function popArgument(args, arg) {
  const index = args.indexOf(arg);
  if (index >= 0) {
    args.splice(index, 1);
  }
  return index >= 0;
}

export function partitionArguments(args, delimiter) {
  const index = args.indexOf(delimiter);
  return index >= 0
    ? [args.slice(0, index), args.slice(index + 1)]
    : [args, []];
}

export async function getInstalledSolanaVersion() {
  try {
    const { stdout } = await $`solana --version`.quiet();
    return stdout.match(/(\d+\.\d+\.\d+)/)?.[1];
  } catch (error) {
    return '';
  }
}
