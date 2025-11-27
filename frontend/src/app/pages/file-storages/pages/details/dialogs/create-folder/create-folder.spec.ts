import { ComponentFixture, TestBed } from '@angular/core/testing';

import { CreateFolder } from './create-folder';

describe('CreateFolder', () => {
  let component: CreateFolder;
  let fixture: ComponentFixture<CreateFolder>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [CreateFolder]
    })
    .compileComponents();

    fixture = TestBed.createComponent(CreateFolder);
    component = fixture.componentInstance;
    await fixture.whenStable();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
