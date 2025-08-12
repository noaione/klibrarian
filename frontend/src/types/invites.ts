export interface InviteSharedLibrary {
  all: boolean;
  libraryIds: string[];
}

export interface InviteOption {
  labelsAllow: string[] | null;
  labelsExclude: string[] | null;
  sharedLibraries: InviteSharedLibrary | null;
  expiresAt: number | null;
  roles: string[] | null;
}

export interface Invite {
  kind: "komga" | "navidrome";
  token: string;
  option: InviteOption;
  user_id: string | null;
}

export interface InviteConfig {
  komga: {
    active: boolean;
    libraries: {
      id: string;
      name: string;
      unavailable: boolean;
    }[];
    labels: string[];
  };
  navidrome: {
    active: boolean;
    libraries: {
      id: number;
      name: string;
    }[];
  };
}

export interface AddEmitKomga {
  mode: "komga";
  libraries: string[];
  labels: string[];
  excludeLabels: string[];
  roles: string[];
  expiresAt?: number | null;
}

export interface AddEmitNavidrome {
  mode: "navidrome";
  libraries: number[];
  isAdmin: boolean;
  expiresAt?: number | null;
}
