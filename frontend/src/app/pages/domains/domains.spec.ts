import { ComponentFixture, TestBed } from '@angular/core/testing';

import { Domains } from './domains';

describe('Domains', () => {
  let component: Domains;
  let fixture: ComponentFixture<Domains>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [Domains]
    })
    .compileComponents();

    fixture = TestBed.createComponent(Domains);
    component = fixture.componentInstance;
    await fixture.whenStable();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
