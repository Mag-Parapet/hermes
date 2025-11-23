import { TestBed } from '@angular/core/testing';

import { Domains } from './domains';

describe('Domains', () => {
  let service: Domains;

  beforeEach(() => {
    TestBed.configureTestingModule({});
    service = TestBed.inject(Domains);
  });

  it('should be created', () => {
    expect(service).toBeTruthy();
  });
});
